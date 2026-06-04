//! Implementation of a heap allocator that uses
//! Cheney's algorithm for garbage collection.

use alloc::vec::Vec;

use crate::values::Value;


/// All allocations into the heap
pub enum HeapItem {
    /// An object which is identified
    /// by some tag and a set of values as fields.
    Object {
        tag: u32,
        fields: Vec<Value>,
    },

    /// A variable-length sequential list of values
    List(Vec<Value>), 
    
    /// This heap item has been forwarded from the From
    /// space to the To space. this is used for the
    /// Cheney's Algorithm garbage collector.
    Forwarded(u32), 
}

