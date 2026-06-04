//! The underlying rust representation of all the differing
//! types of values operatable on in Lepton3

/// All kinds of values in Lepton3
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value {
    /// A unit ()
    Unit,

    /// A simple signed integer
    Int(i64),

    /// A simple floating point number
    Float(f64),

    /// A simple boolean
    Bool(bool),

    /// A unique create-once value which should always be unique up to MAX_u64 (which should realistically be unreachable) 
    Tag(u64),       

    /// Integer handle pointing into the heap at an Object
    Object(u32),    

    /// Integer handle pointing into the heap at a List
    List(u32),      
}