<h1 align="center">Lepton3</h1>

<div align="center">
⚛️🔬⚡🌌🌀
</div>
<div align="center">
  <strong>Object-based Bytecode VM</strong>
</div>
<div align="center">
  A bytecode virtual machine for the <code>Fermion3</code> language.
</div>


## 🌌 Table of Contents
- [<code>✨ What is Lepton3?</code>](#what-is-lepton3)
- [<code>🔭 Community</code>](#community)
- [<code>⚛️ Lepton3 Bytecode</code>](#lepton3-bytecode)
- [<code>🧾 License</code>](#license)
- [<code>🎓 Acknowledgments</code>](#acknowledgements)

<a name="what-is-lepton3"></a>
## ✨ What is Lepton3?

`Lepton3` is an experimental free and open-source bytecode virtual machine for the `Fermion3` language. `Fermion3` aims to be an improvement of the prior `Faerlys` and `Quasar2` languages. As it is part of version 3.0, there is a `3` at the end.

<a name="community"></a>
## 🔭 Community

Before contributing or participating in discussions with the community, you should familiarize yourself with our [**Code of Conduct**](./CODE_OF_CONDUCT.md).

* **[Discord](https://discord.gg/wXzj2cqZ3Q):** Fermion3's official discord server.

If there are any other communities that should be added to the list, please make a PR.

If you'd like to help build Lepton3, check out the **[Contributor's Guide](./CONTRIBUTING.md)**.

<a name="lepton3-bytecode"></a>
## ⚛️ Lepton3 Bytecode

The `Lepton3` virtual machine executes a `Lepton3` format image that contains all of the bytecode instructions. This image must be in the following format (also defined in the `lepton_image` crate):

```
[ HEADER ]
   `magic`:                 [u8; 7]    // "LEPTON3"
   `version_major`:         u8         // major version of lepton3 this is targeted for
   `flags`:                 u16        // flags for the image
   `entry_point`:           u32        // index into function table

[ OBJECT TABLE ]
  `count`:                  u32        // total number of objects
  for each object type:
    `field_count`:          u32        // number of fields of the object

[ FUNCTION TABLE ]
  `count`:                  u32        // total number of functions
  for each function:
    `arg_count`:            u32        // number of arguments to pass to the function
    `local_count`:          u32        // the maximum number of locals in this function
    `instruction_offset`:   u32        // byte offset into instruction stream
    `instruction_length`:   u32        // byte length of this function's instructions

[ INSTRUCTIONS ]
  `length`:                 u32        // total byte length
  `instructions`:           [u8]       // raw instruction stream

[ DEBUG INFO ]                        // only present if flags bit 0 set
  // file table
  `file_count`:             u32
  for each file:
    `length`:               u16
    `bytes`:                [u8]       // utf-8 source file name

  // location table
  `entry_count`:            u32
  for each entry:
    `instruction_offset`:   u32        // sorted ascending
    `file`:                 u32        // index into string table
    `line`:                 u32        // line into the file
    `column`:               u32        // column into the file
```

When running a `Lepton3` image, it is parsed which may lead to these errors:

- The image is malformed and does not contain enough data that is expected.
- The magic bytes do not match "Lepton3".
- A string in the image is not valid UTF-8.

The `Lepton3` image is then validated with some extra checks:

- The major version of the `Lepton3 Virtual Machine` and the image must match. 
- The entry point must be in the bounds of the function table.
- All function's instruction range must be in the instruction stream.
- All function's `local_count` must be greater than its `arg_count`.
- All debug info file indices must be in the bounds of the location table.
- The location table should be sorted by instruction offset, this is important for mapping instructions back to source locations.

The virtual machine will then begin execution at the function specified by the `entry_point` index into the function table.

## Values

`Lepton3` has the following value kinds on which the instructions manipulate:

- `Int`: A 64-bit signed integer. 
- `Float`: a 64-bit IEEE 754 floating point number.
- `Unit`: the unit value (), representing the absence of a value.
- `Tag`: a globally unique opaque identifier.
- `Array`: a heap-allocated vector of values.
- `Boolean`: a `true` or `false` value.
- `Object`: a heap-allocated collection of fields, associated with a unique type Tag.

`Int`, `Float`, `Boolean`, `Unit` and `Tag` are value types. They are stored directly on the stack and copied by value in all operations. Copying a `Tag` does not produce a new unique identifier, but rather copies the same identifier.

`Array` and `Object` are reference types. The stack holds a reference to a heap-allocated value. Instructions such as `Duplicate`, `Load`, `ArrayNth` and `ObjectGet` will copy the reference, not the underlying heap data.

`Object` and `Array` are mutable reference types. `ObjectSet`/`ArraySet` mutates the object/array in place, and all references to that object/array will observe the change.

Heap-allocated values are garbage collected in `Lepton3` and values maybe moved or compacted transparently.

## Instruction Set

All instruction opcodes are `1` byte. They may have inline operands (only the `Push` instructions).

The rest of the instructions receive their arguments from the stack. If a value is expected from the stack but no operand values are available, execution is aborted with a `StackUnderflow` error.

### PushInt (0x00)

This instruction pushes a constant `Int` value onto the stack.

The `PushInt` instruction is an inline-operand instruction which carries both the `PushInt` opcode and the constant value to push onto the stack.

The instruction format is as follows:

```
[ `PushInt`;   1 byte ][ value; 8 bytes ]
```

### PushBool (0x01)

This instruction pushes a constant `Boolean` value onto the stack.

The `PushBool` instruction is an inline-operand instruction which carries both the `PushBool` opcode and the constant value to push onto the stack. Any value which is `!= 0` is considered `true` and any value which is considered `== 0` is considered `false`.

The instruction format is as follows:

```
[ `PushBool`;  1 byte ][ value; 1 byte  ]
```

### PushUnit (0x02)

This instruction pushes a constant `Unit` value onto the stack `()`.

The instruction format is as follows:

```
[ `PushUnit`; 1 byte ]
```

### Duplicate (0x03)

This instruction duplicates the value at the top of the stack.

The instruction format is as follows:

```
[ `Duplicate`; 1 byte ]
```

### Pop (0x04)

This instruction discards the top of the stack.

The instruction format is as follows:

```
[ `Pop`; 1 byte ]
```

### Swap (0x05)

This instruction swaps the top two values of the stack. 

This is such that if the stack is as follows:

```
[ ..., a, b ]
```

The resulting stack after `Swap` is

```
[ ..., b, a ]
```

The instruction format is as follows:

```
[ `Swap`; 1 byte ]
```

### PushFloat (0x06)

This instruction pushes a constant `Float` value onto the stack.

The `PushFloat` instruction is an inline-operand instruction which carries both the `PushFloat` opcode and the constant value to push onto the stack.

The instruction format is as follows:

```
[ `PushFloat`; 1 byte ][ value; 8 bytes ]
```

## Add (0x10)

Pops two `Int` values and pushes their sum. Uses wrapping arithmetic around the boundary of the `Int` type.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a + b ]
```

The instruction format is as follows:

```
[ `Add`; 1 byte ]
```

## Sub (0x11)

Pops two `Int` values and pushes their difference. Uses wrapping arithmetic around the boundary of the `Int` type.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a - b ]
```

The instruction format is as follows:

```
[ `Sub`; 1 byte ]
```

## Mul (0x12)

Pops two `Int` values and pushes their product. Uses wrapping arithmetic around the boundary of the `Int` type.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a * b ]
```

The instruction format is as follows:

```
[ `Mul`; 1 byte ]
```

## Div (0x13)

Pops two `Int` values and pushes their integer quotient. Uses wrapping arithmetic around the boundary of the `Int` type.

Aborts execution with a `DivisionByZero` error if the divisor is `0`

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a / b ]
```

The instruction format is as follows:

```
[ `Div`; 1 byte ]
```

## Mod (0x14)

Pops two `Int` values and pushes their remainder.

Aborts execution with a `ModuloByZero` error if the divisor is `0`

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a % b ]
```

The instruction format is as follows:

```
[ `Mod`; 1 byte ]
```

## Neg (0x15)

Pops one `Int` value and pushes its negation. Uses wrapping arithmetic around the boundary of the `Int` type.

The stack will be modified as follows:

```
[ ..., a ]
```

Will become

```
[ ..., -a ]
```

The instruction format is as follows:

```
[ `Neg`; 1 byte ]
```

## ShiftL (0x16)

Pops two `Int` values and pushes the result of a left shift. Shift amounts >= 64 produce 0.


The right-hand side must be a non-negative value that fits in a u32, otherwise execution is aborted with a `ShiftRHSTooLarge` error. 

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a << b ]
```

The instruction format is as follows:

```
[ `ShiftL`; 1 byte ]
```


## ShiftR (0x17)

Pops two `Int` values and pushes the result of a right shift. Shift amounts >= 64 produce 0.


The right-hand side must be a non-negative value that fits in a u32, otherwise execution is aborted with a `ShiftRHSTooLarge` error. 

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a >> b ]
```

The instruction format is as follows:

```
[ `ShiftR`; 1 byte ]
```

## And (0x18)

Pops two `Int` values and pushes their bitwise AND or `&`. 

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a & b ]
```

The instruction format is as follows:

```
[ `And`; 1 byte ]
```

## Or (0x19)

Pops two `Int` values and pushes their bitwise OR or `|`. 

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a | b ]
```

The instruction format is as follows:

```
[ `Or`; 1 byte ]
```

## Xor (0x1A)

Pops two `Int` values and pushes their bitwise XOR or `⊕`. 

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a ⊕ b ]
```

The instruction format is as follows:

```
[ `Xor`; 1 byte ]
```


## Not (0x1B)

Pops one `Int` value and pushes its bitwise NOT. 

The stack will be modified as follows:

```
[ ..., a ]
```

Will become

```
[ ..., ~a ]
```

The instruction format is as follows:

```
[ `Not`; 1 byte ]
```

## Equal (0x21)

Pops two `Int` values and pushes a `Boolean` indicating whether they are equal.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a == b ]
```

The instruction format is as follows:

```
[ `Equal`; 1 byte ]
```


## NotEqual (0x22)

Pops two `Int` values and pushes a `Boolean` indicating whether they are not equal.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a != b ]
```

The instruction format is as follows:

```
[ `NotEqual`; 1 byte ]
```

## LessThan (0x23)

Pops two `Int` values and pushes a `Boolean` indicating whether the first is less than the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a < b ]
```

The instruction format is as follows:

```
[ `LessThan`; 1 byte ]
```

## LessThanEq (0x24)

Pops two `Int` values and pushes a `Boolean` indicating whether the first is less than or equal the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a <= b ]
```

The instruction format is as follows:

```
[ `LessThanEq`; 1 byte ]
```

## GreaterThan (0x25)

Pops two `Int` values and pushes a `Boolean` indicating whether the first is greater than the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a > b ]
```

The instruction format is as follows:

```
[ `GreaterThan`; 1 byte ]
```

## GreaterThanEq (0x26)

Pops two `Int` values and pushes a `Boolean` indicating whether the first is greater than or equal the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a >= b ]
```

The instruction format is as follows:

```
[ `GreaterThanEq`; 1 byte ]
```

## BoolAnd (0x31)

Pops two `Boolean` values and pushes their logical AND or `&&`.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a && b ]
```

The instruction format is as follows:

```
[ `BoolAnd`; 1 byte ]
```

## BoolOr (0x32)

Pops two `Boolean` values and pushes their logical OR or `||`.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a || b ]
```

The instruction format is as follows:

```
[ `BoolOr`; 1 byte ]
```

## BoolNot (0x33)

Pops one `Boolean` value and pushes its logical NOT or `!`.

The stack will be modified as follows:

```
[ ..., a ]
```

Will become

```
[ ..., !a ]
```

The instruction format is as follows:

```
[ `BoolNot`; 1 byte ]
```

## Jump (0x41)

Pops one `Int` byte offset and unconditionally jumps to that position within the current function's instruction stream. 

This byte offset is an offset from the instruction base of the current function.

The offset must be non-negative, otherwise execution is aborted with an `InvalidIndex` error.

The stack will be modified as follows:

```
[ ..., offset ]
```

Will become

```
[ ... ]
```

The instruction format is as follows:

```
[ `Jump`; 1 byte ]
```


## JumpIfTrue (0x42)

Pops one `Int` byte offset, then pops a `Boolean`.

If the `Boolean` value is `true` then jumps to that position within the current function's instruction stream. 

This byte offset is an offset from the instruction base of the current function.

The offset must be non-negative, otherwise execution is aborted with an `InvalidIndex` error.

The stack will be modified as follows:

```
[ ..., offset, boolean ]
```

Will become

```
[ ... ]
```

The instruction format is as follows:

```
[ `JumpIfTrue`; 1 byte ]
```


## JumpIfFalse (0x43)

Pops one `Int` byte offset, then pops a `Boolean`.

If the `Boolean` value is `false` then jumps to that position within the current function's instruction stream. 

This byte offset is an offset from the instruction base of the current function.

The offset must be non-negative, otherwise execution is aborted with an `InvalidIndex` error.

The stack will be modified as follows:

```
[ ..., offset, boolean ]
```

Will become

```
[ ... ]
```

The instruction format is as follows:

```
[ `JumpIfFalse`; 1 byte ]
```

## Call (0x44)

Pops an `Int` function index and looks up the function in the function table. 

The function's declared argument count `n` is looked up from the function table, and the top `n` values on the stack become the first `n` locals of the new call frame.

Any additional locals beyond the argument count are initialised to Unit.  

Aborts execution with an `InvalidFunction` error if the function index does not exist in the function table.

The stack must at least have `n` values for the arguments, else execution is aborted with an `ArgumentCountMismatch` error.

The stack for the `Call` instruction is:

```
[ ..., local0, ..., localN, func_idx ]
```

The instruction format is as follows:

```
[ `Call`; 1 byte ]
```

## Return (0x45)

Pops the top of the stack as the return value for the current function, tears down the current call frame, and resumes the caller. 

If this was the final call frame, execution halts and the return value is the result of the execution of the image.

The stack for the `Return` instruction is:

```
[ ..., return_value ]
```

In the caller's frame, the stack will be:

```
[ ..., return_value ]
```

The instruction format is as follows:

```
[ `Return`; 1 byte ]
```

## Abort (0x46)

Unconditionally halts execution with an unrecoverable `Abort` error.

The instruction format is as follows:

```
[ `Abort`; 1 byte ]
```

## TailCall (0x47)

Pops an `Int` function index and performs the same as the `Call` instruction.

Instead of growing the call stack with a new call frame, this instruction reuses the current call frame.

The stack for the `TailCall` instruction is:

```
[ ..., local0, ..., localN, func_idx ]
```

The instruction format is as follows:

```
[ `TailCall`; 1 byte ]
```

## Load (0x51)

Pops an `Int` local index from the stack and pushes a copy of that local variable's value onto the stack. 

Aborts execution with a `OutOfBounds` error if the index exceeds the current call frame's local count.

The stack will be modified as follows:

```
[ ..., local_idx ]
```

Will become

```
[ ..., locals[local_idx] ]
```
The instruction format is as follows:

```
[ `Load`; 1 byte ]
```

## Store (0x52)

Pops an `Int` local index from the stack, then pops a value and writes that value into the local variable at the index.

Aborts execution with a `OutOfBounds` error if the index exceeds the current call frame's local count.

The stack should be as follows for the `Store` instruction:

```
[ ..., value, local_idx ]
```

The instruction format is as follows:

```
[ `Store`; 1 byte ]
```

## ArrayNew (0x61)

Pushes a new empty `Array` onto the stack.

The stack after running `ArrayNew` will be:

```
[ ..., [] ]
```

The instruction format is as follows:

```
[ `ArrayNew`; 1 byte ]
```

## ArrayCons (0x62)

Pops a value and then an `Array`, and pushes a new `Array` with the value prepended at index 0.

The stack will be modified as follows:

```
[ ..., array, value ]
```

Will become

```
[ ..., [value, ...array] ]
```

The instruction format is as follows:

```
[ `ArrayCons`; 1 byte ]
```

## ArrayHead (0x63)

Pops an `Array` and pushes its first element. 

Aborts execution with an `OutOfBounds` error if the array is empty.


The stack will be modified as follows:

```
[ ..., array ]
```

Will become

```
[ ..., array[0] ]
```

The instruction format is as follows:

```
[ `ArrayHead`; 1 byte ]
```


## ArrayTail (0x64)

Pops an `Array` and pushes a new `Array` containing all elements except the first.

Aborts execution with an `OutOfBounds` error if the array is empty.

The stack will be modified as follows:

```
[ ..., array ]
```

Will become

```
[ ..., array[1..] ]
```

The instruction format is as follows:

```
[ `ArrayTail`; 1 byte ]
```

## ArrayLength (0x65)

Pops an `Array` and pushes its length as an `Int`. 

If the array's length exceeds the value holdable by an `Int`, execution is aborted with a `ValueTooLarge` error.

The stack will be modified as follows:

```
[ ..., array ]
```

Will become

```
[ ..., len(array) ]
```

The instruction format is as follows:

```
[ `ArrayLength`; 1 byte ]
```

## ArrayNth (0x66)

Pops an `Int` index, then pops an `Array`, and pushes the element at that index. 

Aborts execution with an `OutOfBounds` error if the index is out of range.

The stack will be modified as follows:

```
[ ..., array, index ]
```

Will become

```
[ ..., array[index] ]
```

The instruction format is as follows:

```
[ `ArrayNth`; 1 byte ]
```

## ArrayAppend (0x67)

Pops two `Array` values and pushes a new `Array` that is the concatenation of the first followed by the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., [...a, ...b] ]
```

The instruction format is as follows:

```
[ `ArrayAppend`; 1 byte ]
```

## ArraySet (0x72)

Pops a value, then an `Int` array index, then an `Array`, updates that index in place in the array, and pushes the array back onto the stack.

Aborts execution with an `OutOfBounds` error if the field index is out of range.

The stack will be modified as follows:

```
[ ..., array, field_idx, value ]
```

Will become

```
[ ..., array ]
```

The instruction format is as follows:

```
[ `ArraySet`; 1 byte ]
```

## ObjectNew (0x71)

Pops an `Int` object table index, looks up the field count for that object type, then pops that many values from the stack and pushes a new object of that type with those fields. 

Each object type is associated with a unique `Tag`.

The stack will be modified as follows:

```
[ ..., field0, ..., fieldN, type_idx ]
```

Will become

```
[ ..., object ]
```

The instruction format is as follows:

```
[ `ObjectNew`; 1 byte ]
```

## ObjectSet (0x72)

Pops a value, then an `Int` field index, then an `Object`, mutates that field in place in the object, and pushes the object back onto the stack.

Aborts execution with an `OutOfBounds` error if the field index is out of range.

The stack will be modified as follows:

```
[ ..., object, field_idx, value ]
```

Will become

```
[ ..., object ]
```

The instruction format is as follows:

```
[ `ObjectSet`; 1 byte ]
```

## ObjectGet (0x73)

Pops an `Int` field index, then an `Object`, and pushes the a copy of the value of that field

Aborts execution with an `OutOfBounds` error if the field index is out of range.

The stack will be modified as follows:

```
[ ..., object, field_idx ]
```

Will become

```
[ ..., object.fields[field_idx] ]
```

The instruction format is as follows:

```
[ `ObjectGet`; 1 byte ]
```

## ObjectLength (0x74)

Pops an `Object` and pushes its field count as an `Int`.

The stack will be modified as follows:

```
[ ..., object ]
```

Will become

```
[ ..., field_count ]
```

The instruction format is as follows:

```
[ `ObjectLength`; 1 byte ]
```

## TagEq (0x81)

Pops two `Tag` values and pushes a `Boolean` indicating whether they are equal.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a == b ]
```

The instruction format is as follows:

```
[ `TagEq`; 1 byte ]
```

## TagNew (0x82)

Allocates and pushes a new `Tag`.

The stack will be modified as follows:

```
[ ... ]
```

Will become

```
[ ..., new_tag ]
```

The instruction format is as follows:

```
[ `TagNew`; 1 byte ]
```

## CallCap (0x91)

Pops an `Int` capability index and invokes the registered capability handler at that index. 

The handler has full access to the stack, heap, and tag generator, and may pop arguments and push results directly. 

Aborts execution with an `UnknownCapability` error if no handler is registered at that index.

The stack may be modified in any way by the handler.

The instruction format is as follows:

```
[ `CallCap`; 1 byte ]
```

## Try (0xA1)

Pops an `Int` byte offset and registers an error handler at that offset within the current function's instruction stream (same byte offset as  the `Jump` instructions).

The current call stack depth is saved alongside the handler so that a subsequent `Raise` can unwind correctly.

The stack should be as follows for the `Try` instruction:

```
[ ..., handler_offset ]
```

The instruction format is as follows:

```
[ `Try`; 1 byte ]
```

## EndTry (0xA2)

Removes the most recently registered error handler.

Aborts execution with an `UnhandledEndTry` error  if no handlers are registered.

The instruction format is as follows:

```
[ `EndTry`; 1 byte ]
```

## Raise (0xA3)

Pops a value from the top of the stack as the error value, then pops the most recently registered error handler.

`Raise` will then unwind the call stack back to the depth at which the corresponding `Try` was registered. In a similar fashion to `Return` the value is pushed onto the stack before jumping to the `Try`.

The stack should be as follows for the `Raise` instruction:

```
[ ..., error_value ]
```

The instruction format is as follows:

```
[ `Raise`; 1 byte ]
```


## FAdd (0xB1)

Pops two `Float` values and pushes their sum.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a + b ]
```

The instruction format is as follows:

```
[ `FAdd`; 1 byte ]
```

## FSub (0xB2)

Pops two `Float` values and pushes their difference.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a - b ]
```

The instruction format is as follows:

```
[ `FSub`; 1 byte ]
```

## FMul (0xB3)

Pops two `Float` values and pushes their product.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a * b ]
```

The instruction format is as follows:

```
[ `FMul`; 1 byte ]
```

## FDiv (0xB4)

Pops two `Float` values and pushes their quotient. This follows IEEE 754 and produces `Inf` or `NaN` as appropriate.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a / b ]
```

The instruction format is as follows:

```
[ `FDiv`; 1 byte ]
```

## FMod (0xB5)

Pops two `Float` values and pushes their remainder.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a % b ]
```

The instruction format is as follows:

```
[ `FMod`; 1 byte ]
```

## FNeg (0xB6)

Pops one Float value and pushes its negation.

The stack will be modified as follows:

```
[ ..., a ]
```

Will become

```
[ ..., -a ]
```

The instruction format is as follows:

```
[ `FNeg`; 1 byte ]
```
## FEqual (0xC1)

Pops two `Float` values and pushes a `Boolean` indicating whether they are equal.

This does not have an error margin.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a == b ]
```

The instruction format is as follows:

```
[ `FEqual`; 1 byte ]
```


## FNotEqual (0xC2)

Pops two `Float` values and pushes a `Boolean` indicating whether they are not equal.

This does not have an error margin.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a != b ]
```

The instruction format is as follows:

```
[ `FNotEqual`; 1 byte ]
```

## FLessThan (0xC3)

Pops two `Float` values and pushes a `Boolean` indicating whether the first is less than the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a < b ]
```

The instruction format is as follows:

```
[ `FLessThan`; 1 byte ]
```

## FLessThanEq (0xC4)

Pops two `Float` values and pushes a `Boolean` indicating whether the first is less than or equal the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a <= b ]
```

The instruction format is as follows:

```
[ `FLessThanEq`; 1 byte ]
```

## FGreaterThan (0xC5)

Pops two `Float` values and pushes a `Boolean` indicating whether the first is greater than the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a > b ]
```

The instruction format is as follows:

```
[ `FGreaterThan`; 1 byte ]
```

## FGreaterThanEq (0xC6)

Pops two `Float` values and pushes a `Boolean` indicating whether the first is greater than or equal the second.

The stack will be modified as follows:

```
[ ..., a, b ]
```

Will become

```
[ ..., a >= b ]
```

The instruction format is as follows:

```
[ `FGreaterThanEq`; 1 byte ]
```

## FIsNan (0xC7)

Pops one `Float` value and pushes a `Boolean` indicating whether it is `NaN`.

The stack will be modified as follows:

```
[ ..., a ]
```

Will become

```
[ ..., isNaN(a) ]
```

The instruction format is as follows:

```
[ `FIsNan`; 1 byte ]
```

## IntToFloat (0xD1)

Pops one `Int` value and pushes it converted to a `Float`. 

Note that precision may be lost since an `Int` is 64 bits wide but `Float` has only 52 bits of mantissa.

The stack will be modified as follows:

```
[ ..., a ]
```

Will become

```
[ ..., float(a) ]
```

The instruction format is as follows:

```
[ `IntToFloat`; 1 byte ]
```

## FloatToInt (0xD2)

Pops one `Float` value and pushes it converted to an `Int` by truncation toward zero.

The stack will be modified as follows:

```
[ ..., a ]
```

Will become

```
[ ..., trunc(a) ]
```

The instruction format is as follows:

```
[ `FloatToInt`; 1 byte ]
```

## TypeOf (0xD3)

Peeks at the value at the top of the stack (without consuming it) and pushes a `Tag` identifying its type. 

For `Int`, `Boolean`, `Unit`, `Array`, `Tag` and `Float` values there is one `Tag` per primitive type. for `Object` values the object's own type-unique tag is returned.

The stack will be modified as follows:

```
[ ..., value ]
```

Will become

```
[ ..., value, type_tag ]
```

The instruction format is as follows:

```
[ `TypeOf`; 1 byte ]
```

<a name="license"></a>
## 🧾 License

This repository and all elements of Lepton3 are licensed under AGPLv3. See the `LICENSE` file in the repository root.

Lepton3 will *always* be free and open-source.

<a name="acknowledgements"></a>
## 🎓 Acknowledgments

- Thanks to ``Lean4``, ``Rust`` & ``Haskell`` for inspiration.
- Thank you for reading this README/Learning about Lepton3! 💛
- [No generative AI will ever be used for contributions, see the AI Policy section.](./CONTRIBUTING.md)

<br>

-------------

[**Created by all Contributors to Lepton3**](https://github.com/duplessisaurore/lepton3/graphs/contributors?all=1)

Love for everyone 💛 
