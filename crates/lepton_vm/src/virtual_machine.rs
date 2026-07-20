//! The Lepton3 bytecode virtual machine.
//!
//! Executes a compiled `Image` by interpreting its instruction stream.
//! and functions from the entry point.

use core::error::Error;

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};
use lepton_image::{
    format::{Image, SourceLocation},
    image_trait::{LeptonImage, LeptonSourceLocation},
};
use lepton_opcodes::Opcode;

use crate::{
    capabilities::CapabilityFn,
    heap_allocator::{HeapAllocator, HeapAllocatorImpl, HeapItem},
    tagger::{TagGenerator, TagGeneratorImpl},
    values::{PolymorphicInt, PolymorphicIntPair, Tag, TypeTags, Value},
};

/// Every way execution can error.
#[derive(Debug)]
pub enum VmError<'source_location, SL: LeptonSourceLocation> {
    /// The instruction pointer exceeds the instruction buffer.
    InvalidInstructionPointer(usize),

    /// An invalid value that cannot be used as an index was
    /// passed as an index
    InvalidIndex(u64),

    /// An opcode byte that is not a valid `Opcode`.
    UnknownOpcode(u8),

    /// An instruction tried to pop more values than currently in the stack.
    StackUnderflow,

    /// Tried to fetch an instruction, but there was no current call frame
    CallFrameStackUnderflow,

    /// Argument count mismatch for a function call from stack and the actual
    /// function arguments
    ArgumentCountMismatch { expected: usize, got: usize },

    /// A value of the wrong type was on the stack.
    TypeError { expected: &'static str, got: String },

    /// Division by zero.
    DivisionByZero,

    /// Modulo by zero
    ModuloByZero,

    /// The rhs of the shift is too large.
    ShiftRHSTooLarge(String),

    /// The length/size of this type is too large to be turnable into length
    ValueTooLarge { value_type: &'static str },

    /// An out-of-bounds local access.
    OutOfBounds { index: usize, len: usize },

    /// A call to a function index that does not exist.
    InvalidFunction(usize),

    /// A reference to an object index that does not exist.
    InvalidObject(usize),

    /// A `Raise` was executed but no error handler was installed.
    UnhandledRaise,

    /// An `EndTry` was called but no error handler was installed.
    UnhandledEndTry,

    /// A capability index that is not registered.
    UnknownCapability(usize),

    /// A capability returned an error
    CapabilityError(Box<dyn Error>),

    /// `Abort` opcode was executed.
    Abort,

    /// A runtime error with a captured stack trace attached
    WithTrace {
        error: Box<VmError<'source_location, SL>>,
        trace: Vec<StackTraceFrame<'source_location, SL>>,
    },
}

/// A single frame in a captured stack trace
#[derive(Debug)]
pub struct StackTraceFrame<'source_location, SL: LeptonSourceLocation> {
    /// Index of the function in the function table
    pub function_idx: usize,

    /// The instruction offset within the function at the time of the error.
    /// Points to the instruction that caused the error.
    pub instruction_offset: usize,

    /// Source location if debug info is present in the image.
    pub source_location: Option<&'source_location SL>,
}

/// A registered error handler
pub struct ErrorHandler {
    /// Instruction offset to jump to on Raise (relative to the
    /// `instruction_base` of the frame that registered it).
    pub offset: usize,

    /// Call stack depth when Try was registered. On Raise, we unwind
    /// back to this depth before jumping to the handler.
    pub call_stack_depth: usize,

    /// The `instruction_base` of the frame that registered this handler,
    /// so we jump to the right function's handler offset.
    pub instruction_base: usize,
}

/// The Lepton3 Interpreter/Virtual machine state
///
/// The 'image lifetime defines the lifetime of the *image* the
/// virtual machine is currently executing.
///
/// Capabilities can access anything other than other capabilities,
/// as capabilities should be constructed at creation time by the vm.
pub struct VirtualMachine<
    'image,
    CS = (),
    SL: LeptonSourceLocation = SourceLocation,
    H: HeapAllocator = HeapAllocatorImpl,
    T: TagGenerator = TagGeneratorImpl,
    I: LeptonImage<SL> = Image,
> {
    /// The image being exectued
    pub image: &'image I,

    /// The current stack of values
    pub stack: Vec<Value>,

    /// The allocator for heap values and GC
    pub heap: H,

    /// The generator for unique tags
    pub tagger: T,

    /// Registered capability handlers.
    capabilities: Vec<CapabilityFn<'image, CS, SL, H, T, I>>,

    /// Records for activations of functions in a stack
    pub call_stack: Vec<CallFrame>,

    /// Registered error handlers for `Try` and `Raise`
    pub error_handlers: Vec<ErrorHandler>,

    /// The current globals set for the VM
    pub globals: Vec<Value>,

    // Pre-allocated well-known type tags.
    pub type_tags: TypeTags,

    // The state shared between all capabilities
    pub capability_state: CS,
}

/// One record for the call of a function
pub struct CallFrame {
    /// Index into the function table of the `Function` being executed.
    pub function_idx: usize,

    /// Byte offset within `instructions` where this function starts.
    pub instruction_base: usize,

    /// Current instruction offset relative to `instruction_base`.
    pub instruction_pointer: usize,

    /// The operand stack index at which this frame's locals begin.
    ///
    /// Locals are stored directly on the value stack below the
    /// operand area for this frame.
    pub locals_base: usize,

    /// Number of local slots (including parameters).
    pub local_count: usize,
}

impl<'image, CS, SL: LeptonSourceLocation, H: HeapAllocator, T: TagGenerator, I: LeptonImage<SL>>
    VirtualMachine<'image, CS, SL, H, T, I>
{
    /// Creates a new VM from an image and a set of capabilities along
    /// with the heap allocator and tag generator.
    ///
    /// Expects the image has been already validated by the `validator`
    pub fn new(
        image: &'image I,
        capabilities: Vec<CapabilityFn<'image, CS, SL, H, T, I>>,
        heap: H,
        mut tagger: T,
        initial_state: CS,
    ) -> Self {
        // Preallocate all object tags so we don't waste time during
        // execution having to consider making a tag or not.
        let obj_tags: Vec<Tag> = (0..image.object_table().len())
            .map(|_| tagger.allocate_tag())
            .collect();

        Self {
            stack: Vec::new(),
            heap,
            type_tags: TypeTags::new(&mut tagger, obj_tags),
            tagger,
            capabilities,
            call_stack: Vec::new(),
            error_handlers: Vec::new(),
            image,
            globals: Vec::new(),
            capability_state: initial_state,
        }
    }

    /// Execute the image starting from its declared entry point.
    ///
    /// Returns the top-of-stack value when the entry-point function returns,
    /// or `Value::Unit` if the stack is empty at that point.
    ///
    /// # Errors
    ///
    /// If something during the execution of the program fails, then
    /// an error will occur. View all the possible execution fails in `VmError`.
    pub fn run(&mut self) -> Result<Value, VmError<'image, SL>> {
        // Call the entry point function
        let entry = self.image.header().entry_point as usize;
        self.call_function(entry, 0)?;

        // Execute until completion
        loop {
            match self.step() {
                Ok(Some(value)) => return Ok(value),
                Ok(None) => {}

                // Capture trace for the error so we have debug info attached.
                Err(error) => {
                    let trace = self.capture_trace();
                    return Err(VmError::WithTrace {
                        error: Box::new(error),
                        trace,
                    });
                }
            }
        }
    }

    /// Push a new call frame for `function_idx`.
    ///
    /// `arg_count` values are expected to already be on the stack.
    /// They become the first `arg_count` locals of the new frame.
    /// Additional locals beyond the argument count are zeroed out to `Value::Unit`
    pub fn call_function(
        &mut self,
        function_idx: usize,
        arg_count: usize,
    ) -> Result<(), VmError<'image, SL>> {
        // Ensure it exists in the function table.
        let func = self
            .image
            .function_table()
            .get(function_idx)
            .ok_or(VmError::InvalidFunction(function_idx))?;

        // Grab the instruction base/etc from the table
        let instruction_base = func.instruction_offset as usize;
        let local_count = func.local_count as usize;
        let declared_args = func.arg_count as usize;

        // Ensure we don't underflow if the caller passed the wrong count.
        if self.stack.len() < arg_count {
            return Err(VmError::ArgumentCountMismatch {
                expected: arg_count,
                got: self.stack.len(),
            });
        }

        // locals_base points at the first argument, which is already on the stack
        let locals_base = self.stack.len() - arg_count;

        // Pad locals beyond the argument count with `Value::Unit`
        let extra = local_count.saturating_sub(declared_args);
        for _ in 0..extra {
            self.stack.push(Value::Unit);
        }

        // Add this function's call frame to the call stack
        self.call_stack.push(CallFrame {
            function_idx,
            instruction_base,
            instruction_pointer: 0,
            locals_base,
            local_count,
        });

        Ok(())
    }

    /// Reuse the current frame for a tail call.
    ///
    /// The new arguments must already be on the stack above the current
    /// frame's locals. They are moved down to overwrite the old locals,
    /// the frame is reset to the new function, and execution continues
    /// from `instruction_pointer=0` without pushing a new `CallFrame`.
    fn tail_call_function(&mut self, function_idx: usize) -> Result<(), VmError<'image, SL>> {
        // Read the function from the function table to get all the new params
        let func = self
            .image
            .function_table()
            .get(function_idx)
            .ok_or(VmError::InvalidFunction(function_idx))?;

        let new_arg_count = func.arg_count as usize;
        let new_local_count = func.local_count as usize;
        let new_instruction_base = func.instruction_offset as usize;

        // New tail called function does not have enough arguments on the stack
        // to give to function
        if self.stack.len() < new_arg_count {
            return Err(VmError::ArgumentCountMismatch {
                expected: new_arg_count,
                got: self.stack.len(),
            });
        }

        // Grab the new arguments off the top of the stack before truncating.
        let args_start = self.stack.len() - new_arg_count;
        let args: Vec<Value> = self.stack.drain(args_start..).collect();

        // Rewind to this frame's `locals_base`, discarding the old locals.
        let locals_base = self.current_frame().locals_base;
        self.stack.truncate(locals_base);

        // Write the new arguments as the first locals of the reused frame.
        self.stack.extend(args);

        // Pad any remaining locals beyond the argument count with Unit.
        let extra = new_local_count.saturating_sub(new_arg_count);
        for _ in 0..extra {
            self.stack.push(Value::Unit);
        }

        // Overwrite the current call frame in place
        let frame = self.current_frame_mut();
        frame.function_idx = function_idx;
        frame.instruction_base = new_instruction_base;
        frame.instruction_pointer = 0;
        frame.local_count = new_local_count;
        Ok(())
    }

    /// Pop the current call frame and clean up its locals from the stack,
    /// leaving exactly one return value or `Value::Unit` on stack.
    fn return_from_function(&mut self, return_value: Value) {
        let frame = self
            .call_stack
            .pop()
            // Shouldn't be possible
            .expect("return_from_function called with empty call stack");

        // Trim everything belonging to this frame's locals from the stack.
        self.stack.truncate(frame.locals_base);

        // Push the return value.
        self.stack.push(return_value);
    }

    /// Execute one instruction.
    ///
    /// Returns:
    /// - `Ok(None)` meaning continue executing,
    /// - `Ok(Some(value))` meaning the top-level entry function has returned `value`,
    /// - `Err(error)` meaning a runtime error occurred with the `error` err.
    ///
    /// # Errors
    ///
    /// Returns an error when some runtime issue has occured during the execution
    /// of an opcode. View `VmError` for the possible error variants.
    #[allow(clippy::too_many_lines)]
    fn step(&mut self) -> Result<Option<Value>, VmError<'image, SL>> {
        // fetch the next opcode and advance ip
        let opcode_byte = self.fetch_byte()?;
        let opcode = Opcode::try_from(opcode_byte).map_err(VmError::UnknownOpcode)?;

        // decode and execute the opcode
        match opcode {
            // = Stack-Modifying Instructions 0x1 =
            Opcode::PushInt => {
                let v = self.fetch_i64()?;
                self.stack.push(Value::Int(v));
            }
            #[cfg(feature = "floats")]
            Opcode::PushFloat => {
                let v = self.fetch_f64()?;
                self.stack.push(Value::Float(v));
            }
            Opcode::PushBool => {
                let b = self.fetch_byte()?;
                self.stack.push(Value::Bool(b != 0));
            }
            Opcode::PushUnit => {
                self.stack.push(Value::Unit);
            }
            Opcode::PushUInt => {
                let v = self.fetch_u64()?;
                self.stack.push(Value::UInt(v));
            }
            Opcode::Duplicate => {
                let v = self.peek().ok_or(VmError::StackUnderflow).copied()?;
                self.stack.push(v);
            }
            Opcode::Pop => {
                self.pop()?;
            }
            Opcode::Swap => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.stack.push(a);
                self.stack.push(b);
            }

            // = Integer arithmetic 0x2 =
            Opcode::Add => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => Value::Int(a.wrapping_add(b)),
                    PolymorphicIntPair::UInt(a, b) => Value::UInt(a.wrapping_add(b)),
                };

                self.stack.push(result);
            }
            Opcode::Sub => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => Value::Int(a.wrapping_sub(b)),
                    PolymorphicIntPair::UInt(a, b) => Value::UInt(a.wrapping_sub(b)),
                };
                self.stack.push(result);
            }
            Opcode::Mul => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => Value::Int(a.wrapping_mul(b)),
                    PolymorphicIntPair::UInt(a, b) => Value::UInt(a.wrapping_mul(b)),
                };
                self.stack.push(result);
            }
            Opcode::Div => {
                let (a, b) = self.pop2_int()?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                self.stack.push(Value::Int(a.wrapping_div(b)));
            }
            Opcode::UDiv => {
                let (a, b) = self.pop2_uint()?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                self.stack.push(Value::UInt(a.wrapping_div(b)));
            }
            Opcode::Mod => {
                let (a, b) = self.pop2_int()?;
                if b == 0 {
                    return Err(VmError::ModuloByZero);
                }
                self.stack.push(Value::Int(a.wrapping_rem(b)));
            }
            Opcode::UMod => {
                let (a, b) = self.pop2_uint()?;
                if b == 0 {
                    return Err(VmError::ModuloByZero);
                }
                self.stack.push(Value::UInt(a.wrapping_rem(b)));
            }
            Opcode::Neg => {
                let a = self.pop_int()?;
                self.stack.push(Value::Int(a.wrapping_neg()));
            }

            // = Bitwise Operations 0x1 =
            Opcode::ShiftL => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => {
                        let rhs = u32::try_from(b)
                            .map_err(|_| VmError::ShiftRHSTooLarge(b.to_string()))?;
                        Value::Int(a.unbounded_shl(rhs))
                    }
                    PolymorphicIntPair::UInt(a, b) => {
                        let rhs = u32::try_from(b)
                            .map_err(|_| VmError::ShiftRHSTooLarge(b.to_string()))?;
                        Value::UInt(a.unbounded_shl(rhs))
                    }
                };
                self.stack.push(result);
            }
            Opcode::ShiftR => {
                let (a, b) = self.pop2_int()?;
                let rhs = u32::try_from(b).map_err(|_| VmError::ShiftRHSTooLarge(b.to_string()))?;
                self.stack.push(Value::Int(a.unbounded_shr(rhs)));
            }
            Opcode::UShiftR => {
                let (a, b) = self.pop2_uint()?;
                let rhs = u32::try_from(b).map_err(|_| VmError::ShiftRHSTooLarge(b.to_string()))?;
                self.stack.push(Value::UInt(a.unbounded_shr(rhs)));
            }
            Opcode::And => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => Value::Int(a & b),
                    PolymorphicIntPair::UInt(a, b) => Value::UInt(a & b),
                };
                self.stack.push(result);
            }
            Opcode::Or => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => Value::Int(a | b),
                    PolymorphicIntPair::UInt(a, b) => Value::UInt(a | b),
                };
                self.stack.push(result);
            }
            Opcode::Xor => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => Value::Int(a ^ b),
                    PolymorphicIntPair::UInt(a, b) => Value::UInt(a ^ b),
                };
                self.stack.push(result);
            }
            Opcode::Not => {
                let result = match self.pop_polymorphic_int()? {
                    PolymorphicInt::Int(a) => Value::Int(!a),
                    PolymorphicInt::UInt(a) => Value::UInt(!a),
                };
                self.stack.push(result);
            }

            // = Integer comparison 0x2 =
            Opcode::Equal => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => Value::Bool(a == b),
                    PolymorphicIntPair::UInt(a, b) => Value::Bool(a == b),
                };
                self.stack.push(result);
            }
            Opcode::NotEqual => {
                let result = match self.pop2_polymorphic_int()? {
                    PolymorphicIntPair::Int(a, b) => Value::Bool(a != b),
                    PolymorphicIntPair::UInt(a, b) => Value::Bool(a != b),
                };
                self.stack.push(result);
            }
            Opcode::LessThan => {
                let (a, b) = self.pop2_int()?;
                self.stack.push(Value::Bool(a < b));
            }
            Opcode::LessThanEq => {
                let (a, b) = self.pop2_int()?;
                self.stack.push(Value::Bool(a <= b));
            }
            Opcode::GreaterThan => {
                let (a, b) = self.pop2_int()?;
                self.stack.push(Value::Bool(a > b));
            }
            Opcode::GreaterThanEq => {
                let (a, b) = self.pop2_int()?;
                self.stack.push(Value::Bool(a >= b));
            }

            // U<Comparison> can have different behaviour from Int to UInt
            // since Int has negatives the first bit is important
            Opcode::ULessThan => {
                let (a, b) = self.pop2_uint()?;
                self.stack.push(Value::Bool(a < b));
            }
            Opcode::ULessThanEq => {
                let (a, b) = self.pop2_uint()?;
                self.stack.push(Value::Bool(a <= b));
            }
            Opcode::UGreaterThan => {
                let (a, b) = self.pop2_uint()?;
                self.stack.push(Value::Bool(a > b));
            }
            Opcode::UGreaterThanEq => {
                let (a, b) = self.pop2_uint()?;
                self.stack.push(Value::Bool(a >= b));
            }

            // = Boolean operations 0x3 =
            Opcode::BoolAnd => {
                let (a, b) = self.pop2_bool()?;
                self.stack.push(Value::Bool(a && b));
            }
            Opcode::BoolOr => {
                let (a, b) = self.pop2_bool()?;
                self.stack.push(Value::Bool(a || b));
            }
            Opcode::BoolNot => {
                let a = self.pop_bool()?;
                self.stack.push(Value::Bool(!a));
            }

            // = Control flow 0x4 =
            Opcode::Jump => {
                let offset = self.pop_index()?;
                self.current_frame_mut().instruction_pointer = offset;
            }
            Opcode::JumpIfTrue => {
                let offset = self.pop_index()?;
                let cond = self.pop_bool()?;
                if cond {
                    self.current_frame_mut().instruction_pointer = offset;
                }
            }
            Opcode::JumpIfFalse => {
                let offset = self.pop_index()?;
                let cond = self.pop_bool()?;
                if !cond {
                    self.current_frame_mut().instruction_pointer = offset;
                }
            }
            Opcode::Call => {
                let func_idx = self.pop_index()?;

                // Get the function from the arguments
                let func = self
                    .image
                    .function_table()
                    .get(func_idx)
                    .ok_or(VmError::InvalidFunction(func_idx))?;
                let arg_count = func.arg_count as usize;

                self.call_function(func_idx, arg_count)?;
            }
            Opcode::TailCall => {
                let func_idx = self.pop_index()?;

                // Tail call the function at the index.
                self.tail_call_function(func_idx)?;
            }
            Opcode::Return => {
                // The return value sits on top of the operand stack.
                let ret = self.pop()?;

                if self.call_stack.len() == 1 {
                    // Returning from the entry-point function.
                    self.call_stack.pop();
                    return Ok(Some(ret));
                }

                self.return_from_function(ret);
            }
            Opcode::Abort => {
                return Err(VmError::Abort);
            }

            // = Locals & Globals 0x5 =
            Opcode::Load => {
                // Get the index of the local in the current stack
                let local_idx = self.pop_index()?;
                let frame = self.current_frame();
                let abs_idx = frame.locals_base + local_idx;

                if local_idx >= frame.local_count {
                    return Err(VmError::OutOfBounds {
                        index: local_idx,
                        len: frame.local_count,
                    });
                }

                let v = self.stack[abs_idx];
                self.stack.push(v);
            }
            Opcode::Store => {
                // Get the index to store the local in the current stack
                let local_idx = self.pop_index()?;
                let value = self.pop()?;

                let (locals_base, local_count) = {
                    let f = self.current_frame();
                    (f.locals_base, f.local_count)
                };

                if local_idx >= local_count {
                    return Err(VmError::OutOfBounds {
                        index: local_idx,
                        len: local_count,
                    });
                }

                self.stack[locals_base + local_idx] = value;
            }
            Opcode::LoadGlobal => {
                // Get the index to load the global from
                let global_idx = self.pop_index()?;

                // Make sure it exists else OOB
                let v = self
                    .globals
                    .get(global_idx)
                    .copied()
                    .ok_or(VmError::OutOfBounds {
                        index: global_idx,
                        len: self.globals.len(),
                    })?;

                self.stack.push(v);
            }
            Opcode::StoreGlobal => {
                // Get the index to store the global in the current stack
                let global_idx = self.pop_index()?;
                let value = self.pop()?;

                // Extend the globals if we are storing into a new location
                // We trust that the user will not explode memory with unlimited globals.
                if global_idx >= self.globals.len() {
                    self.globals.resize(global_idx + 1, Value::Unit);
                }

                self.globals[global_idx] = value;
            }

            // = Array operations 0x6 =
            Opcode::ArrayNew => {
                // Collect to make space for a new array if necessary
                // and update roots.
                self.gc_collect();

                // The number of values this array should begin with
                let array_length = self.pop_index()?;
                let mut initial_values = Vec::with_capacity(array_length);

                // Pop all the values from the stack into the array
                for _ in 0..array_length {
                    initial_values.push(self.pop()?);
                }
                initial_values.reverse();

                let idx = self.heap.alloc_raw(HeapItem::Array(initial_values));
                self.stack.push(Value::Array(idx));
            }
            Opcode::ArrayCons => {
                self.gc_collect();
                let value = self.pop()?;
                let arr_idx = self.pop_array()?;

                // Clone the existing array
                let mut new_items = match &self.heap.get_item(arr_idx) {
                    HeapItem::Array(v) => v.clone(),
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: arr_idx,
                            len: 0,
                        });
                    }
                };

                // Prepend the new value
                new_items.insert(0, value);
                let new_idx = self.heap.alloc_raw(HeapItem::Array(new_items));
                self.stack.push(Value::Array(new_idx));
            }
            Opcode::ArrayHead => {
                let arr_idx = self.pop_array()?;

                // Clone the first item in the array and push to the stack
                let head = match &self.heap.get_item(arr_idx) {
                    HeapItem::Array(v) => v
                        .first()
                        .copied()
                        .ok_or(VmError::OutOfBounds { index: 0, len: 0 })?,
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: arr_idx,
                            len: 0,
                        });
                    }
                };

                self.stack.push(head);
            }
            Opcode::ArrayTail => {
                self.gc_collect();
                let arr_idx = self.pop_array()?;

                // Get all of the elements after the first and clone
                let tail: Vec<Value> = match &self.heap.get_item(arr_idx) {
                    HeapItem::Array(v) => {
                        if v.is_empty() {
                            return Err(VmError::OutOfBounds { index: 0, len: 0 });
                        }
                        v[1..].to_vec()
                    }
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: arr_idx,
                            len: 0,
                        });
                    }
                };

                let new_idx = self.heap.alloc_raw(HeapItem::Array(tail));
                self.stack.push(Value::Array(new_idx));
            }
            Opcode::ArrayLength => {
                let arr_idx = self.pop_array()?;

                // Push the length of the array as an i64 onto the stack
                let len = match &self.heap.get_item(arr_idx) {
                    HeapItem::Array(v) => {
                        u64::try_from(v.len()).map_err(|_| VmError::ValueTooLarge {
                            value_type: "Array",
                        })?
                    }
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: arr_idx,
                            len: 0,
                        });
                    }
                };

                self.stack.push(Value::UInt(len));
            }
            Opcode::ArrayNth => {
                let n = self.pop_index()?;
                let arr_idx = self.pop_array()?;

                // Get the element from the position and copy it onto the stack
                let elem = match &self.heap.get_item(arr_idx) {
                    HeapItem::Array(v) => v.get(n).copied().ok_or(VmError::OutOfBounds {
                        index: n,
                        len: v.len(),
                    })?,
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: arr_idx,
                            len: 0,
                        });
                    }
                };

                self.stack.push(elem);
            }
            Opcode::ArrayAppend => {
                self.gc_collect();

                let b_idx = self.pop_array()?;
                let a_idx = self.pop_array()?;

                // Get both the array from a and b and combine them
                let mut combined = match &self.heap.get_item(a_idx) {
                    HeapItem::Array(v) => v.clone(),
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: a_idx,
                            len: 0,
                        });
                    }
                };

                let b_items = match &self.heap.get_item(b_idx) {
                    HeapItem::Array(v) => v.clone(),
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: b_idx,
                            len: 0,
                        });
                    }
                };
                combined.extend(b_items);

                // Allocate the new combined array
                let new_idx = self.heap.alloc_raw(HeapItem::Array(combined));
                self.stack.push(Value::Array(new_idx));
            }
            Opcode::ArraySet => {
                // Pop the value, field and then array
                let value = self.pop()?;
                let field_idx = self.pop_index()?;
                let arr_idx = self.pop_array()?;

                // Modify the item
                match self.heap.get_item_mut(arr_idx) {
                    HeapItem::Array(values) => {
                        let len = values.len();
                        *values.get_mut(field_idx).ok_or(VmError::OutOfBounds {
                            index: field_idx,
                            len,
                        })? = value;
                    }
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: arr_idx,
                            len: 0,
                        });
                    }
                }

                self.stack.push(Value::Array(arr_idx));
            }

            // = Object operations 0x7 =
            Opcode::ObjectNew => {
                self.gc_collect();

                // The type index comes first to get the number of fields
                // to pop
                let type_idx = self.pop_index()?;

                // Get the number of fields from the table
                let field_count = self
                    .image
                    .object_table()
                    .get(type_idx)
                    .ok_or(VmError::InvalidObject(type_idx))?
                    .field_count as usize;

                // Pop all the fields from the stack
                let mut fields: Vec<Value> = Vec::with_capacity(field_count);
                for _ in 0..field_count {
                    fields.push(self.pop()?);
                }
                fields.reverse();

                // Get the unique tag associated with this object type.
                //
                // This is preallocated on the start of the VM
                let tag = self.type_tags.object[type_idx];

                // Allocate in the heap
                let idx = self.heap.alloc_raw(HeapItem::Object { tag, fields });

                self.stack.push(Value::Object(idx));
            }
            Opcode::ObjectSet => {
                // Pop the value, field and then object
                let value = self.pop()?;
                let field_idx = self.pop_index()?;
                let obj_idx = self.pop_object()?;

                // Modify the item
                match self.heap.get_item_mut(obj_idx) {
                    HeapItem::Object { fields, .. } => {
                        let len = fields.len();
                        *fields.get_mut(field_idx).ok_or(VmError::OutOfBounds {
                            index: field_idx,
                            len,
                        })? = value;
                    }
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: obj_idx,
                            len: 0,
                        });
                    }
                }

                self.stack.push(Value::Object(obj_idx));
            }
            Opcode::ObjectGet => {
                // Pop the field index, object index and then
                // move the value onto the heap
                let field_idx = self.pop_index()?;
                let obj_idx = self.pop_object()?;

                let v = match &self.heap.get_item(obj_idx) {
                    HeapItem::Object { fields, .. } => {
                        let len = fields.len();
                        fields.get(field_idx).copied().ok_or(VmError::OutOfBounds {
                            index: field_idx,
                            len,
                        })?
                    }
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: obj_idx,
                            len: 0,
                        });
                    }
                };

                self.stack.push(v);
            }
            Opcode::ObjectLength => {
                let obj_idx = self.pop_object()?;
                let len = match &self.heap.get_item(obj_idx) {
                    HeapItem::Object { fields, .. } => {
                        u64::try_from(fields.len()).map_err(|_| VmError::ValueTooLarge {
                            value_type: "Object",
                        })?
                    }
                    _ => {
                        return Err(VmError::OutOfBounds {
                            index: obj_idx,
                            len: 0,
                        });
                    }
                };
                self.stack.push(Value::UInt(len));
            }
            Opcode::ObjectTypeTag => {
                // The type index to retrieve the tag of
                let type_idx = self.pop_index()?;

                // Get the unique tag associated with this object type.
                //
                // This is preallocated on the start of the VM, which
                // will either exist or not depending on the validity of the
                // `type_idx` here.
                let tag = self
                    .type_tags
                    .object
                    .get(type_idx)
                    .ok_or(VmError::InvalidObject(type_idx))?;

                // Push tag onto the stack..
                self.stack.push(Value::Tag(*tag));
            }

            // = Tag operations 0x8 =
            Opcode::TagEq => {
                // The inner value of the tag must be equal
                let b = self.pop_tag()?;
                let a = self.pop_tag()?;
                self.stack.push(Value::Bool(a == b));
            }
            Opcode::TagNew => {
                // Allocate a new tag
                self.stack.push(Value::Tag(self.tagger.allocate_tag()));
            }

            // = Capability operations 0x9 =
            Opcode::CallCap => {
                // Get the handler from the capability index.
                let cap_idx = self.pop_index()?;

                let handler = self
                    .capabilities
                    .get(cap_idx)
                    .copied()
                    .ok_or(VmError::UnknownCapability(cap_idx))?;

                // Call it for some result.
                handler(self)?;
            }

            // = Error handling 0xA =
            Opcode::Try => {
                // Push a new error handler onto the list of error handlers.
                let handler_offset = self.pop_index()?;
                let frame = self.current_frame();

                self.error_handlers.push(ErrorHandler {
                    offset: handler_offset,
                    call_stack_depth: self.call_stack.len(),
                    instruction_base: frame.instruction_base,
                });
            }
            Opcode::EndTry => {
                // Remove the last error handler frame
                self.error_handlers.pop().ok_or(VmError::UnhandledEndTry)?;
            }
            Opcode::Raise => {
                let error_value = self.pop()?;
                let handler = self.error_handlers.pop().ok_or(VmError::UnhandledRaise)?;

                // Unwind the call stack back to where Try was registered,
                // restoring the value stack for each popped frame.
                while self.call_stack.len() > handler.call_stack_depth {
                    let frame = self.call_stack.pop().unwrap();
                    self.stack.truncate(frame.locals_base);
                }

                // Truncate any operands left over in the surviving frame's operands
                let locals_top = {
                    let f = self.current_frame();
                    f.locals_base + f.local_count
                };
                self.stack.truncate(locals_top);
                self.stack.push(error_value);

                // Jump into the handler, we need to reset the instruction base and stuff
                // into when it was registered to be correct.
                let frame = self.current_frame_mut();
                frame.instruction_base = handler.instruction_base;
                frame.instruction_pointer = handler.offset;
            }

            // = Floating point arithmetic 0xB =
            #[cfg(feature = "floats")]
            Opcode::FAdd => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Float(a + b));
            }
            #[cfg(feature = "floats")]
            Opcode::FSub => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Float(a - b));
            }
            #[cfg(feature = "floats")]
            Opcode::FMul => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Float(a * b));
            }
            #[cfg(feature = "floats")]
            Opcode::FDiv => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Float(a / b));
            }
            #[cfg(feature = "floats")]
            Opcode::FNeg => {
                let a = self.pop_float()?;
                self.stack.push(Value::Float(-a));
            }
            #[cfg(feature = "floats")]
            Opcode::FMod => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Float(a % b));
            }

            // = Floating point comparison 0xC =
            #[cfg(feature = "floats")]
            Opcode::FEqual => {
                let (a, b) = self.pop2_float()?;

                // This is intended to be strict comparison,
                // if the user wants to have a margin of error
                // then they should implement it themselves.
                #[allow(clippy::float_cmp)]
                self.stack.push(Value::Bool(a == b));
            }
            #[cfg(feature = "floats")]
            Opcode::FNotEqual => {
                let (a, b) = self.pop2_float()?;
                #[allow(clippy::float_cmp)]
                self.stack.push(Value::Bool(a != b));
            }
            #[cfg(feature = "floats")]
            Opcode::FLessThan => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Bool(a < b));
            }
            #[cfg(feature = "floats")]
            Opcode::FLessThanEq => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Bool(a <= b));
            }
            #[cfg(feature = "floats")]
            Opcode::FGreaterThan => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Bool(a > b));
            }
            #[cfg(feature = "floats")]
            Opcode::FGreaterThanEq => {
                let (a, b) = self.pop2_float()?;
                self.stack.push(Value::Bool(a >= b));
            }
            #[cfg(feature = "floats")]
            Opcode::FIsNaN => {
                let a = self.pop_float()?;
                self.stack.push(Value::Bool(a.is_nan()));
            }

            // = Type conversion 0xD =
            #[cfg(feature = "floats")]
            Opcode::IntToFloat => {
                let i = self.pop_int()?;

                // Precision loss is known, part of the opcodes spec
                #[allow(clippy::cast_precision_loss)]
                self.stack.push(Value::Float(i as f64));
            }
            #[cfg(feature = "floats")]
            Opcode::FloatToInt => {
                let f = self.pop_float()?;

                // Truncation is intentional
                #[allow(clippy::cast_possible_truncation)]
                self.stack.push(Value::Int(f as i64));
            }
            #[cfg(feature = "floats")]
            Opcode::UIntToFloat => {
                let i = self.pop_uint()?;

                // Precision loss is known, part of the opcodes spec
                #[allow(clippy::cast_precision_loss)]
                self.stack.push(Value::Float(i as f64));
            }
            #[cfg(feature = "floats")]
            Opcode::FloatToUInt => {
                let f = self.pop_float()?;

                // Truncation is intentional and so is the sign loss.
                #[allow(clippy::cast_possible_truncation)]
                #[allow(clippy::cast_sign_loss)]
                self.stack.push(Value::UInt(f as u64));
            }
            Opcode::IntToUInt => {
                let i = self.pop_int()?;
                self.stack.push(Value::UInt(i.cast_unsigned()));
            }
            Opcode::UIntToInt => {
                let i = self.pop_uint()?;
                self.stack.push(Value::Int(i.cast_signed()));
            }
            Opcode::TypeOf => {
                // Get the tag for the value
                let type_tag = match self.peek().ok_or(VmError::StackUnderflow)? {
                    Value::Unit => self.type_tags.unit,
                    Value::Int(_) => self.type_tags.int,
                    Value::Float(_) => self.type_tags.float,
                    Value::Bool(_) => self.type_tags.boolean,
                    Value::Tag(_) => self.type_tags.tag,
                    Value::Array(_) => self.type_tags.array,
                    Value::UInt(_) => self.type_tags.uint,
                    Value::Object(obj_idx) => {
                        // For an object, we return it's tag.
                        match &self.heap.get_item(*obj_idx) {
                            HeapItem::Object { tag, .. } => *tag,
                            _ => {
                                return Err(VmError::OutOfBounds {
                                    index: *obj_idx,
                                    len: 0,
                                });
                            }
                        }
                    }
                };

                // Push it to the stack.
                self.stack.push(Value::Tag(type_tag));
            }
            // = Heap Operations 0xE =
            Opcode::Clone => {
                self.gc_collect();
                let value = *self.peek().ok_or(VmError::StackUnderflow)?;

                let cloned = match value {
                    // Value types are the same as `Dup`, no allocation needed.
                    Value::Unit
                    | Value::Int(_)
                    | Value::UInt(_)
                    | Value::Bool(_)
                    | Value::Tag(_) => value,
                    #[cfg(feature = "floats")]
                    Value::Float(_) => value,

                    // Heap reference types
                    Value::Array(arr_idx) => {
                        // Allocate a new array with all the values cloned.
                        let cloned_items = {
                            match self.heap.get_item(arr_idx) {
                                HeapItem::Array(v) => v.clone(),
                                _ => {
                                    return Err(VmError::OutOfBounds {
                                        index: arr_idx,
                                        len: 0,
                                    });
                                }
                            }
                        };
                        let new_idx = self.heap.alloc_raw(HeapItem::Array(cloned_items));
                        Value::Array(new_idx)
                    }
                    Value::Object(obj_idx) => {
                        // Allocate a new object with all the fields cloned.
                        let (tag, fields) = {
                            match self.heap.get_item(obj_idx) {
                                HeapItem::Object { tag, fields } => (*tag, fields.clone()),
                                _ => {
                                    return Err(VmError::OutOfBounds {
                                        index: obj_idx,
                                        len: 0,
                                    });
                                }
                            }
                        };
                        let new_idx = self.heap.alloc_raw(HeapItem::Object { tag, fields });
                        Value::Object(new_idx)
                    }
                };

                self.stack.push(cloned);
            }
        }

        Ok(None)
    }

    /// Returns the current call frame
    /// Expects that at least one exists.
    fn current_frame(&self) -> &CallFrame {
        self.call_stack.last().expect("call stack is empty")
    }

    /// Returns a mutable reference to the current call frame
    /// expects that at least one exists
    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.call_stack.last_mut().expect("call stack is empty")
    }

    /// Expect one byte from the current instruction stream and advance the
    /// `instruction_pointer` of the current call frame past this byte.
    fn fetch_byte(&mut self) -> Result<u8, VmError<'image, SL>> {
        let frame = self
            .call_stack
            .last_mut()
            .ok_or(VmError::CallFrameStackUnderflow)?;

        // Get the position of the current instruction
        let abs = frame.instruction_base + frame.instruction_pointer;

        // Read it from the image
        let byte = self
            .image
            .instructions()
            .get(abs)
            .copied()
            .ok_or(VmError::InvalidInstructionPointer(abs))?;

        // Advance the instruction pointer so the next fetch returns the next instruction
        frame.instruction_pointer += 1;
        Ok(byte)
    }

    /// Expect a little-endian `i64` from the instruction stream.
    fn fetch_i64(&mut self) -> Result<i64, VmError<'image, SL>> {
        let mut buf = [0u8; 8];
        for b in &mut buf {
            *b = self.fetch_byte()?;
        }
        Ok(i64::from_le_bytes(buf))
    }

    /// Expect a little-endian `u64` from the instruction stream.
    fn fetch_u64(&mut self) -> Result<u64, VmError<'image, SL>> {
        let mut buf = [0u8; 8];
        for b in &mut buf {
            *b = self.fetch_byte()?;
        }
        Ok(u64::from_le_bytes(buf))
    }

    /// Expect a little-endian `f64` from the instruction stream.
    #[cfg(feature = "floats")]
    fn fetch_f64(&mut self) -> Result<f64, VmError<'image, SL>> {
        let mut buf = [0u8; 8];
        for b in &mut buf {
            *b = self.fetch_byte()?;
        }
        Ok(f64::from_le_bytes(buf))
    }

    /// Pop a value from the top of the stack
    fn pop(&mut self) -> Result<Value, VmError<'image, SL>> {
        // We must not pop into the locals area of the current frame.
        let locals_top = self
            .call_stack
            .last()
            .map_or(0, |f| f.locals_base + f.local_count);

        // No more left to pop, stack underflow
        if self.stack.len() <= locals_top {
            return Err(VmError::StackUnderflow);
        }

        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    /// Peeks at the last value in the stack
    fn peek(&self) -> Option<&Value> {
        self.stack.last()
    }

    /// Expects to convert an `u64` popped from the stack into a `usize` index,
    /// returning `InvalidIndex` as the error if the value is not allowed.
    fn pop_index(&mut self) -> Result<usize, VmError<'image, SL>> {
        let i = self.pop_uint()?;
        usize::try_from(i).map_err(|_| VmError::InvalidIndex(i))
    }

    /// Expects a uint at the top of the stack and pops it off
    fn pop_uint(&mut self) -> Result<u64, VmError<'image, SL>> {
        match self.pop()? {
            Value::UInt(u) => Ok(u),
            other => Err(VmError::TypeError {
                expected: "UInt",
                got: String::from(value_type_name(&other)),
            }),
        }
    }

    /// Expects a int at the top of the stack and pops it off
    fn pop_int(&mut self) -> Result<i64, VmError<'image, SL>> {
        match self.pop()? {
            Value::Int(i) => Ok(i),
            other => Err(VmError::TypeError {
                expected: "Int",
                got: String::from(value_type_name(&other)),
            }),
        }
    }

    /// Expects a int or unsigned int at the top of the stack and pops it off
    fn pop_polymorphic_int(&mut self) -> Result<PolymorphicInt, VmError<'image, SL>> {
        let a = self.pop()?;

        match a {
            // Supported are i64, i64 and u64, u64
            Value::Int(x) => Ok(PolymorphicInt::Int(x)),
            Value::UInt(x) => Ok(PolymorphicInt::UInt(x)),

            _ => Err(VmError::TypeError {
                expected: "Int or UInt Pair",
                got: String::from(value_type_name(&a)),
            }),
        }
    }

    /// Expects a float at the top of the stack and pops it off
    #[cfg(feature = "floats")]
    fn pop_float(&mut self) -> Result<f64, VmError<'image, SL>> {
        match self.pop()? {
            Value::Float(f) => Ok(f),
            other => Err(VmError::TypeError {
                expected: "Float",
                got: String::from(value_type_name(&other)),
            }),
        }
    }

    /// Expects a bool at the top of the stack and pops it off
    fn pop_bool(&mut self) -> Result<bool, VmError<'image, SL>> {
        match self.pop()? {
            Value::Bool(b) => Ok(b),
            other => Err(VmError::TypeError {
                expected: "Bool",
                got: String::from(value_type_name(&other)),
            }),
        }
    }

    /// Expects an array at the top of the stack and pops it off
    fn pop_array(&mut self) -> Result<usize, VmError<'image, SL>> {
        match self.pop()? {
            Value::Array(idx) => Ok(idx),
            other => Err(VmError::TypeError {
                expected: "Array",
                got: String::from(value_type_name(&other)),
            }),
        }
    }

    /// Expects an object at the top of the stack and pops it off
    fn pop_object(&mut self) -> Result<usize, VmError<'image, SL>> {
        match self.pop()? {
            Value::Object(idx) => Ok(idx),
            other => Err(VmError::TypeError {
                expected: "Object",
                got: String::from(value_type_name(&other)),
            }),
        }
    }

    /// Expects a tag at the top of the stack and pops it off
    fn pop_tag(&mut self) -> Result<crate::values::Tag, VmError<'image, SL>> {
        match self.pop()? {
            Value::Tag(t) => Ok(t),
            other => Err(VmError::TypeError {
                expected: "Tag",
                got: String::from(value_type_name(&other)),
            }),
        }
    }

    /// Expects two integers at the top of the stack and pops them off
    fn pop2_int(&mut self) -> Result<(i64, i64), VmError<'image, SL>> {
        let b = self.pop_int()?;
        let a = self.pop_int()?;
        Ok((a, b))
    }

    /// Expects two uints at the top of the stack and pops them off
    fn pop2_uint(&mut self) -> Result<(u64, u64), VmError<'image, SL>> {
        let b = self.pop_uint()?;
        let a = self.pop_uint()?;
        Ok((a, b))
    }

    /// Pops two values and ensures they are matching integer types.
    /// Returns an enum indicating which type of integers were popped.
    fn pop2_polymorphic_int(&mut self) -> Result<PolymorphicIntPair, VmError<'image, SL>> {
        let b = self.pop()?;
        let a = self.pop()?;

        match (a, b) {
            // Supported are i64, i64 and u64, u64
            (Value::Int(x), Value::Int(y)) => Ok(PolymorphicIntPair::Int(x, y)),
            (Value::UInt(x), Value::UInt(y)) => Ok(PolymorphicIntPair::UInt(x, y)),

            // Mixed integer types (not allowed)
            (Value::Int(_), Value::UInt(_)) | (Value::UInt(_), Value::Int(_)) => {
                Err(VmError::TypeError {
                    expected: "matching integer types",
                    got: format!("mixed {} and {}", value_type_name(&a), value_type_name(&b)),
                })
            }
            _ => Err(VmError::TypeError {
                expected: "Int or UInt Pair",
                got: format!("{} and {} Pair", value_type_name(&a), value_type_name(&b)),
            }),
        }
    }

    /// Expects two floats at the top of the stack and pops them off
    #[cfg(feature = "floats")]
    fn pop2_float(&mut self) -> Result<(f64, f64), VmError<'image, SL>> {
        let b = self.pop_float()?;
        let a = self.pop_float()?;
        Ok((a, b))
    }

    /// Expects two bools at the top of the stack and pops them off
    fn pop2_bool(&mut self) -> Result<(bool, bool), VmError<'image, SL>> {
        let b = self.pop_bool()?;
        let a = self.pop_bool()?;
        Ok((a, b))
    }

    /// Run a GC collection cycle, using all stack and global values as roots.
    ///
    /// Read the `ensure_capacity` function of the `HeapAllocator` for
    /// an important note
    fn gc_collect(&mut self) {
        let mut refs: Vec<&mut Value> = self
            .stack
            .iter_mut()
            .chain(self.globals.iter_mut())
            .collect();
        self.heap.ensure_capacity(refs.as_mut_slice());
    }

    /// Captures the current call stack as a stack trace, resolving
    /// source locations from debug info if available.
    ///
    /// The trace is ordered most-recent frame first.
    fn capture_trace(&self) -> Vec<StackTraceFrame<'image, SL>> {
        self.call_stack
            .iter()
            .rev()
            .map(|frame| {
                // `instruction_pointer` has already been advanced past the
                // current instruction by `fetch_byte`, so subtract 1 to
                // point back at the instruction that actually failed.
                let instruction_offset = frame.instruction_pointer.saturating_sub(1);

                // Ignore an invalid offset with source info lookup
                // to try still get as good debug info as we can
                let Ok(abs_offset) = u32::try_from(frame.instruction_base + instruction_offset)
                else {
                    return StackTraceFrame {
                        function_idx: frame.function_idx,
                        instruction_offset,
                        source_location: None,
                    };
                };

                // Resolve the source location if debug info is attached
                let source_location = self.resolve_source_location(abs_offset);

                // Add a stack trace frame for this call frame
                StackTraceFrame {
                    function_idx: frame.function_idx,
                    instruction_offset,
                    source_location,
                }
            })
            .collect()
    }

    /// Resolves an absolute instruction offset to a source location
    /// by searching the debug info table
    ///
    /// Returns `None` if no location covers the given offset.
    fn resolve_source_location(&self, abs_offset: u32) -> Option<&'image SL> {
        let Some(locations) = &self.image.locations() else {
            return None;
        };

        // Find the closest location which covers this instruction
        let idx = locations
            .partition_point(|loc| loc.instruction_offset() <= abs_offset)
            .saturating_sub(1);

        locations.get(idx)
    }
}

// Name for the value for debugging
fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Unit => "Unit",
        Value::Int(_) => "Int",
        Value::UInt(_) => "UInt",
        Value::Float(_) => "Float",
        Value::Bool(_) => "Bool",
        Value::Tag(_) => "Tag",
        Value::Object(_) => "Object",
        Value::Array(_) => "Array",
    }
}

impl<SL: LeptonSourceLocation> From<Box<dyn Error>> for VmError<'_, SL> {
    fn from(value: Box<dyn Error>) -> Self {
        Self::CapabilityError(value)
    }
}
