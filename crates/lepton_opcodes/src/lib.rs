//! `Lepton3` is an experimental free and open-source bytecode virtual
//! machine for the `Fermion3` language.
//!
//! Check out the [repository README](https://github.com/duplessisaurore/lepton3/blob/main/README.md)
//! for more information about the project and join the [Discord](https://discord.gg/wXzj2cqZ3Q) for
//! any discussion.
//!
//! ## Lepton3 Opcodes
//!
//! The `lepton3_opcodes` crate provides the set of operations
//! and their opcodes for execution in the VM

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
        pub enum Opcode {
            $($(#[$attr])* $name = $val),*
        }

        impl Opcode {
            pub fn operand_size(&self) -> u8 {
                match self {
                    $(Self::$name => $args,)*
                }
            }
        }

        impl TryFrom<u8> for Opcode {
            type Error = u8;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($val => Ok(Self::$name),)*
                    _ => Err(value),
                }
            }
        }
    };
}

opcode_enum! {
    // Stack based operations 0x0
    /// Pushes an integer constant onto the stack.
    PushInt = (0x00, 8),

    /// Pushes a boolean constant onto the stack.
    PushBool = (0x01, 1),

    /// Pushes a UNIT () onto the stack.
    PushUnit = (0x02, 0),

    /// Duplicates the top of the stack.
    Duplicate = (0x03, 0),

    /// Discards the top of the stack
    Pop = (0x04, 0),

    /// Swaps the top two values of the stack.
    Swap = (0x05, 0),

    /// Pushes a floating point constant onto the stack
    PushFloat = (0x06, 8),

    // Integer Arithmetic 0x1
    /// +
    Add = (0x10, 0),

    /// -
    Sub = (0x11, 0),

    /// *
    Mul = (0x12, 0),

    /// //
    Div = (0x13, 0),

    /// %
    Mod = (0x14, 0),

    /// Integer negation (-)
    Neg = (0x15, 0),

    /// Bitwise operations

    /// Left shift <<
    ShiftL = (0x16, 0),

    /// Right shift >>
    ShiftR = (0x17, 0),

    /// Bitwise AND
    And = (0x18, 0),

    /// Bitwise OR
    Or = (0x19, 0),

    /// Bitwise XOR
    Xor = (0x1A, 0),

    /// Bitwise NOT
    Not = (0x1B, 0),

    // Comparison Operators
    /// Integer comparison 0x2

    /// =
    Equal = (0x21, 0),

    /// <>
    NotEqual = (0x22, 0),

    /// <
    LessThan = (0x23, 0),

    /// <=
    LessThanEq = (0x24, 0),

    /// >
    GreaterThan = (0x25, 0),

    /// >=
    GreaterThanEq = (0x26, 0),

    // Boolean Comparison 0x3
    /// Logical AND &&
    BoolAnd = (0x31, 0),

    /// Logical OR ||
    BoolOr = (0x32, 0),

    /// Logical NOT !
    BoolNot = (0x33, 0),

    // Control flow 0x4
    /// Jumps to a byte offset within the current function's instruction stream.
    Jump = (0x41, 4),

    /// Pops a boolean off the stack and jumps to a byte offset within
    /// the current function's instruction stream if true.
    JumpIfTrue = (0x42, 4),

    /// Pops a boolean off the stack and jumps to a byte offset within
    /// the current function's instruction stream if false.
    JumpIfFalse = (0x43, 4),

    // Calls a function based on an index into the function table
    Call = (0x44, 4),

    // Returns from the current function back to the caller
    Return = (0x45, 0),

    // An unrecoverable error which should halt all execution
    Abort = (0x46, 0),

    // Locals 0x5
    /// Loads a local from some index and pushes it onto the stack
    Load = (0x51, 4),

    /// Pops from the stack and puts the value into some index in the locals
    Store = (0x52, 4),

    // List Operations 0x6
    /// Pushes a new list onto the stack
    ListNew = (0x61, 0),

    /// Pops a value and list from the stack, pushes a new list with the value prepended
    ListCons = (0x62, 0),

    /// Pop a list and push first element onto stack
    ListHead = (0x63, 0),

    // Pop a list and push list without first element
    ListTail = (0x64, 0),

    /// Pop a list and push its length onto the stack
    ListLength = (0x65, 0),

    /// Pop a list and int index, push element at index onto the stack
    ListNth = (0x66, 0),

    /// Pop two lists and push the concatenated list
    ListAppend = (0x67, 0),

    // Object operations 0x7

    /// Creates a new object based on an index into the object table
    /// Pop values from stack for object fields and push object with tag layout
    ObjectNew = (0x71, 4),

    /// Pop an object and value and return a new object with the field at the
    /// operand index set to the value
    ObjectSet = (0x72, 4),

    // Pop object, push field at index
    ObjectGet = (0x73, 4),

    /// Pop an object and push its field count
    ObjectLength = (0x74, 0),

    // Tag Operations 0x8
    /// Pop two tags and push a boolean based on their equality
    TagEq = (0x81, 0),

    /// Pushes a new tag onto the stack
    TagNew = (0x82, 0),

    // Capability Operations 0x9
    /// Invoke a core capability identified by an integer
    CallCap = (0x91, 4),

    // Error handling 0xA
    /// Registers an error handler at a byte offset within the current
    /// function's instruction stream.
    Try = (0xA1, 4),

    /// Pop the last registered error handler
    EndTry = (0xA2, 0),

    /// Pops the last registered error handler
    /// and jumps to the error handler's offset
    ///
    /// Aborts if no error handler registered
    Raise = (0xA3, 0),

    // Floating point operations
    /// +
    FAdd = (0xB1, 0),

    /// -
    FSub = (0xB2, 0),

    /// *
    FMul = (0xB3, 0),

    /// /
    FDiv = (0xB4, 0),

    /// Floating point negation (-)
    FNeg = (0xB5, 0),

    /// %
    FMod = (0xB6, 0),

    // Floating point comparison 0xC
    /// =
    FEqual = (0xC1, 0),

    /// <>
    FNotEqual = (0xC2, 0),

    /// <
    FLessThan = (0xC3, 0),

    /// <=
    FLessThanEq = (0xC4, 0),

    /// >
    FGreaterThan = (0xC5, 0),

    /// >=
    FGreaterThanEq = (0xC6, 0),

    /// = NaN
    FIsNaN = (0xC7, 0),

    // Type based helpers 0xD
    IntToFloat = (0xD1, 0),
    FloatToInt = (0xD2, 0),

    /// Pushes a tag that identifies the type of the element
    /// at the top of the stack
    TypeOf = (0xD3, 0)
}
