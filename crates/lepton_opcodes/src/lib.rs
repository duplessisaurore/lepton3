//! `Lepton3` is an experimental free and open-source bytecode virtual
//! machine for the `Fermion3` language.
//!
//! Check out the [repository README](https://github.com/duplessisaurore/lepton3/blob/main/README.md)
//! for more information about the project and join the [Discord](https://discord.gg/wXzj2cqZ3Q) for
//! any discussion.
//!
//! ## Lepton3 Opcodes
//!
//! The `lepton_opcodes` crate provides the set of operations
//! and their opcodes for execution in the VM.
//!
//! ## Instruction Format
//!
//! All instructions are a single opcode byte. There are no inline operands
//! except for the three Push instructions which carry their constant value
//! inline:
//!
//!   [ `PushInt`;   1 byte ][ value; 8 bytes ]
//!   [ `PushFloat`; 1 byte ][ value; 8 bytes ]
//!   [ `PushBool`;  1 byte ][ value; 1 byte  ]
//!
//! All other instructions pop their arguments from the stack for uniformity.

#![warn(clippy::pedantic)]
#![no_std]

///
/// All of the opcodes of the language correspond
/// to some operation in the virtual machine.
///
/// Generally an instruction in the VM is some
/// [ opcode; 1 byte ] [ operand; <operand-size> ]
///
/// This `opcode_enum` macro outputs the opcodes as an enum
/// with a `TryFrom<u8>` impl and a `operand_size` from pairs of:
///
/// <EnumVariantName> = (<Opcode>, <OperandSize>),
///
macro_rules! opcode_enum {
    ($($(#[$attr:meta])* $name:ident = ($val:expr, $args:expr)),* $(,)?) => {
        #[repr(u8)]
        #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
        pub enum Opcode {
            $($(#[$attr])* $name = $val),*
        }

        impl Opcode {
            pub fn operand_size(&self) -> u8 {
                #[allow(unused_doc_comments)]
                match self {
                    $($(#[$attr])* Self::$name => $args,)*
                }
            }
        }

        impl TryFrom<u8> for Opcode {
            type Error = u8;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                #[allow(unused_doc_comments)]
                match value {
                    $($(#[$attr])* $val => Ok(Self::$name),)*
                    _ => Err(value),
                }
            }
        }
    };
}

opcode_enum! {
    // Stack based operations 0x0

    /// Pushes an integer constant onto the stack.
    /// [ `PushInt`; 1 byte ][ value; 8 bytes ]
    PushInt = (0x00, 8),

    /// Pushes a boolean constant onto the stack.
    /// [ `PushBool`; 1 byte ][ value; 1 byte ]
    PushBool = (0x01, 1),

    /// Pushes a UNIT () onto the stack.
    PushUnit = (0x02, 0),

    /// Duplicates the top of the stack.
    Duplicate = (0x03, 0),

    /// Discards the top of the stack.
    Pop = (0x04, 0),

    /// Swaps the top two values of the stack.
    Swap = (0x05, 0),

    /// Pushes a floating point constant onto the stack.
    /// [ `PushFloat`; 1 byte ][ value; 8 bytes ]
    #[cfg(feature = "floats")]
    PushFloat = (0x06, 8),

    /// Pushes an unsigned integer constant onto the stack.
    /// [ `PushUInt`; 1 byte ][ value; 8 bytes ]
    PushUInt = (0x07, 8),

    // Integer Arithmetic 0x1

    /// Pops two integers and pushes their sum.
    Add = (0x10, 0),

    /// Pops two integers and pushes their difference.
    Sub = (0x11, 0),

    /// Pops two integers and pushes their product.
    Mul = (0x12, 0),

    /// Pops two integers and pushes their integer quotient.
    ///
    /// This is not compatible with unsigned integers
    Div = (0x13, 0),

    /// Pops two integers and pushes their remainder.
    ///
    /// This is not compatible with unsigned integers
    Mod = (0x14, 0),

    /// Pops an integer and pushes its negation.
    Neg = (0x15, 0),

    // Bitwise operations

    /// Pops two integers and pushes the result of a left shift.
    ShiftL = (0x16, 0),

    /// Pops two integers and pushes the result of a right shift.
    ///
    /// This is not unsigned integer compatible
    ShiftR = (0x17, 0),

    /// Pops two integers and pushes their bitwise AND.
    And = (0x18, 0),

    /// Pops two integers and pushes their bitwise OR.
    Or = (0x19, 0),

    /// Pops two integers and pushes their bitwise XOR.
    Xor = (0x1A, 0),

    /// Pops an integer and pushes its bitwise NOT.
    Not = (0x1B, 0),

    /// Pops two unsigned integers and pushes their integer quotient.
    UDiv = (0x1C, 0),

    /// Pops two unsigned integers and pushes their integer quotient.
    UMod = (0x1D, 0),

    /// Pops two unsigned integers and pushes the result of a right shift.
    UShiftR = (0x1E, 0),

    // Integer Comparison 0x2

    /// Pops two integers and pushes whether they are equal.
    Equal = (0x21, 0),

    /// Pops two integers and pushes whether they are not equal.
    NotEqual = (0x22, 0),

    /// Pops two integers and pushes whether the first is less than the second.
    ///
    /// This is not unsigned integer compatible
    LessThan = (0x23, 0),

    /// Pops two integers and pushes whether the first is less than or equal to the second.
    ///
    /// This is not unsigned integer compatible
    LessThanEq = (0x24, 0),

    /// Pops two integers and pushes whether the first is greater than the second.
    ///
    /// This is not unsigned integer compatible
    GreaterThan = (0x25, 0),

    /// Pops two integers and pushes whether the first is greater than or equal to the second.
    ///
    /// This is not unsigned integer compatible
    GreaterThanEq = (0x26, 0),

    /// Pops two unsigned integers and pushes whether the first is less than the second.
    ULessThan = (0x27, 0),

    /// Pops two unsigned integers and pushes whether the first is less than or equal to the second.
    ULessThanEq = (0x28, 0),

    /// Pops two unsigned integers and pushes whether the first is greater than the second.
    UGreaterThan = (0x29, 0),

    /// Pops two unsigned integers and pushes whether the first is greater than or equal to the second.
    UGreaterThanEq = (0x2A, 0),

    // Boolean Operations 0x3

    /// Pops two booleans and pushes their logical AND.
    BoolAnd = (0x31, 0),

    /// Pops two booleans and pushes their logical OR.
    BoolOr = (0x32, 0),

    /// Pops a boolean and pushes its logical NOT.
    BoolNot = (0x33, 0),

    // Control Flow 0x4

    /// Pops an integer byte offset and jumps to that position
    /// within the current function's instruction stream.
    Jump = (0x41, 0),

    /// Pops an integer byte offset, then pops a boolean.
    /// Jumps to the offset if the boolean is true.
    JumpIfTrue = (0x42, 0),

    /// Pops an integer byte offset, then pops a boolean.
    /// Jumps to the offset if the boolean is false.
    JumpIfFalse = (0x43, 0),

    /// Pops an integer function index and calls that function.
    Call = (0x44, 0),

    /// Returns from the current function back to the caller.
    Return = (0x45, 0),

    /// An unrecoverable error which halts all execution.
    Abort = (0x46, 0),

    /// Tail-calls a function, reusing the current stack frame.
    ///
    /// Similar to Call but does not push a new call frame onto the
    /// call stack.
    TailCall = (0x47, 0),

    // Locals & Globals 0x5

    /// Pops an integer local index and pushes the value of that local
    /// onto the stack.
    Load = (0x51, 0),

    /// Pops an integer local index and a value and stores the value
    /// into that local.
    Store = (0x52, 0),

    /// Pops an integer global index and pushes the value of that global
    /// onto the stack.
    LoadGlobal = (0x53, 0),

    /// Pops an integer global index and a value and stores the value
    /// into that global.
    StoreGlobal = (0x54, 0),

    // Array Operations 0x6

    /// Pops a `UInt` length of initial values, and all of those initial
    /// values.
    ///
    /// Pushes a new `Array` onto the stack filled with those values.
    ///
    /// These values are popped in reverse order (last value first).
    ///
    /// The initial `UInt` length does not constrain the array's total length
    /// and it can freely grow.
    ArrayNew = (0x61, 0),

    /// Pops a value and an array and pushes a new array
    /// with the value prepended.
    ArrayCons = (0x62, 0),

    /// Pops an array and pushes its first element.
    ArrayHead = (0x63, 0),

    /// Pops an array and pushes a new array without its first element.
    ArrayTail = (0x64, 0),

    /// Pops an array and pushes its length.
    ArrayLength = (0x65, 0),

    /// Pops an integer index, then pops an array and pushes the element at that index.
    ArrayNth = (0x66, 0),

    /// Pops two arrays and pushes their concatenation.
    ArrayAppend = (0x67, 0),

    /// Pops a value, an integer array index and an array and pushes
    /// a new array with that index updated to the new value in place
    ArraySet = (0x68, 0),

    // Object Operations 0x7

    /// Pops an integer object type index and field values from the stack
    /// and pushes a new object of that type.
    /// Fields are popped in reverse order (last field first).
    ObjectNew = (0x71, 0),

    /// Pops a value, an integer field index and an object and pushes
    /// a new object with that field set to the value.
    ObjectSet = (0x72, 0),

    /// Pops an integer field index and an object and pushes
    /// the value of that field.
    ObjectGet = (0x73, 0),

    /// Pops an object and pushes its field count.
    ObjectLength = (0x74, 0),

    /// Pops an integer object type index and pushes its specific
    /// unique tag for that type
    ObjectTypeTag = (0x75, 0),

    // Tag Operations 0x8

    /// Pops two tags and pushes whether they are equal.
    TagEq = (0x81, 0),

    /// Pushes a new unique tag onto the stack.
    TagNew = (0x82, 0),

    // Capability Operations 0x9

    /// Pops an integer capability index and invokes that capability.
    CallCap = (0x91, 0),

    // Error Handling 0xA

    /// Pops an integer byte offset and registers an error handler
    /// at that offset within the current function's instruction stream.
    Try = (0xA1, 0),

    /// Pops the last registered error handler.
    EndTry = (0xA2, 0),

    /// Pops the last registered error handler and jumps to its offset.
    /// Aborts if no error handler is registered.
    Raise = (0xA3, 0),

    // Floating Point Operations 0xB

    /// Pops two floats and pushes their sum.
    #[cfg(feature = "floats")]
    FAdd = (0xB1, 0),

    /// Pops two floats and pushes their difference.
    #[cfg(feature = "floats")]
    FSub = (0xB2, 0),

    /// Pops two floats and pushes their product.
    #[cfg(feature = "floats")]
    FMul = (0xB3, 0),

    /// Pops two floats and pushes their quotient.
    #[cfg(feature = "floats")]
    FDiv = (0xB4, 0),

    /// Pops two floats and pushes their remainder.
    #[cfg(feature = "floats")]
    FMod = (0xB5, 0),

    /// Pops a float and pushes its negation.
    #[cfg(feature = "floats")]
    FNeg = (0xB6, 0),

    // Floating Point Comparison 0xC

    /// Pops two floats and pushes whether they are equal.
    #[cfg(feature = "floats")]
    FEqual = (0xC1, 0),

    /// Pops two floats and pushes whether they are not equal.
    #[cfg(feature = "floats")]
    FNotEqual = (0xC2, 0),

    /// Pops two floats and pushes whether the first is less than the second.
    #[cfg(feature = "floats")]
    FLessThan = (0xC3, 0),

    /// Pops two floats and pushes whether the first is less than or equal to the second.
    #[cfg(feature = "floats")]
    FLessThanEq = (0xC4, 0),

    /// Pops two floats and pushes whether the first is greater than the second.
    #[cfg(feature = "floats")]
    FGreaterThan = (0xC5, 0),

    /// Pops two floats and pushes whether the first is greater than or equal to the second.
    #[cfg(feature = "floats")]
    FGreaterThanEq = (0xC6, 0),

    /// Pops a float and pushes whether it is NaN.
    #[cfg(feature = "floats")]
    FIsNaN = (0xC7, 0),

    // Type Conversion 0xD

    /// Pops an integer and pushes it converted to a float.
    /// precision maybe lost as i64 is 64 bits wide, but f64
    /// is only 52 bits wide
    #[cfg(feature = "floats")]
    IntToFloat = (0xD1, 0),

    /// Pops a float and pushes it converted to an integer by truncation.
    #[cfg(feature = "floats")]
    FloatToInt = (0xD2, 0),

    /// Pops an unsigned integer and pushes it converted to a float.
    /// Same issues as `IntToFloat`
    #[cfg(feature = "floats")]
    UIntToFloat = (0xD3, 0),

    /// Pops a float and pushes it converted to an unsigned integer by truncation.
    #[cfg(feature = "floats")]
    FloatToUInt = (0xD4, 0),

    /// Pops an integer and attempts to converts it to an unsigned integer
    IntToUInt = (0xD5, 0),

    /// Pops an unsigned integer and attempts to converts it to an integer
    UIntToInt = (0xD6, 0),

    /// Pushes a tag identifying the type of the value at the top of the stack.
    /// Does not consume the value.
    TypeOf = (0xD8, 0),

    // = Heap Operations 0xE =

    /// Clones the top value on the stack.
    ///
    /// If the value is a reference to a heap value, the heap value
    /// is cloned, therefore producing a new reference to a heap value.
    ///
    /// If the value is a "value-type" value which is not a reference,
    /// it is copied (similar to dup).
    ///
    /// The new value/reference is then pushed onto the stack.
    Clone = (0xE1, 0)
}
