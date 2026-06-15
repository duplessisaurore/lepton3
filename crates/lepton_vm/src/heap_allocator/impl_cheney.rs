//! Implementation of a heap allocator that uses
//! Cheney's algorithm for garbage collection.

use alloc::vec::Vec;

use crate::{
    heap_allocator::{HeapAllocator, HeapItem},
    values::Value,
};

pub struct CheneyAllocator {
    // Using the Cheney memory management model
    // with semispaces
    /// One of the possible semispaces (from)
    pub from_space: Vec<HeapItem>,

    /// One of the possible semispaces (to)
    pub to_space: Vec<HeapItem>,

    /// The threshold at which the garbage
    /// collector should begin collecting
    pub gc_threshold: usize,

    /// The utilisation threshold out of 100
    /// for which we consider the heap
    /// cramped
    pub cramped_threshold: usize,

    /// The utilisation threshold out of 100
    /// for which we consider as wasting ram
    /// e.g too low usage
    pub wasting_threshold: usize,

    /// The minimum heap size to protect the
    /// size from shrinking to zero or a very low
    /// value which would cause a rapid GC cycle
    pub min_heap_size: usize,

    /// The goal for the utilisation of the total
    /// heap size as (1 / target) of the heap size
    pub target_utilisation: usize,
}

impl CheneyAllocator {
    /// Creates a new Cheney semi-space allocator with an initial capacity.
    ///
    /// cramped and wasting threshold should both be between 0 and 100.
    ///
    /// the cramped threshold indicates the % at which above we consider the heap cramped
    /// The wasting threshold is the same but under we consider the heap as too big and wasting space.
    /// The target utilisation is the goal for utilisation as 1 / target of the heap size
    ///
    /// The minimum reserved amount is the minimum amount we limit the GC from shrinking the heap to,
    /// this should be sufficiently high that we dont have to immediately suffer GC on a small heap.
    #[must_use]
    pub fn new(
        initial_threshold: usize,
        cramped_threshold: usize,
        wasting_threshold: usize,
        target_utilisation: usize,
        min_heap_size: usize,
    ) -> Self {
        Self {
            from_space: Vec::with_capacity(initial_threshold),
            to_space: Vec::with_capacity(initial_threshold),
            gc_threshold: initial_threshold,
            cramped_threshold,
            wasting_threshold,
            target_utilisation,
            min_heap_size,
        }
    }

    /// Moves all reachable objects from `from_space` into `to_space` and updates pointers.
    fn collect(&mut self, roots: &mut [&mut Value]) {
        // Clear out the target space to prepare for the moving
        self.to_space.clear();

        // Migrate all of the roots passed from the VM directly into
        // the `to_space`
        for root in roots {
            self.migrate_value(root);
        }

        // Because heap values and arrays can refer to other heap items
        // we need to bring them over into our new `to_space`
        //
        // so while we havent scanned the entire `to_space`'s values internal
        // fields referenced items, keep scanning for any references we may have missed.
        let mut scan_idx = 0;
        while scan_idx < self.to_space.len() {
            // pull the item temporarily out of the `to_space``
            // so we can loop over its internal fields without having to
            // borrow `to_space`.
            let mut item = core::mem::replace(&mut self.to_space[scan_idx], HeapItem::Forwarded(0));

            match &mut item {
                HeapItem::Object { fields, .. } => {
                    // Migrate all of the fields over from an object
                    for val in fields {
                        self.migrate_value(val);
                    }
                }
                HeapItem::Array(fields) => {
                    // Migrate all of the fields over from an array
                    for val in fields {
                        self.migrate_value(val);
                    }
                }
                HeapItem::Forwarded(_) => {
                    unreachable!("The queue should never contain a forwarded indicator")
                }
            }

            // Put the item back into its slot in `to_space`
            self.to_space[scan_idx] = item;
            scan_idx += 1;
        }

        // Swap the spaces, so our heap now uses the new `to_space` values,
        // and clear the old heap, which will drop all of the things we no longer
        // refer to.
        core::mem::swap(&mut self.from_space, &mut self.to_space);
        self.to_space.clear();

        // Calculate how many hepa items actually survived
        let live_items = self.from_space.len();

        // Calculate the utilisation percentage.
        let current_utilisation = (live_items * 100)
            .checked_div(self.gc_threshold)
            .unwrap_or(100);

        // change the threshold of the next gc based on the current utilisation
        if current_utilisation > self.cramped_threshold {
            // Expand the current gc threshold so that our live items take up
            // more like target utilisation amount of space.
            self.gc_threshold =
                core::cmp::max(live_items * self.target_utilisation, self.min_heap_size);
        } else if current_utilisation < self.wasting_threshold {
            // Shrink the spaces so that our live items take up more
            // like the target utilisation amount of space in the heap.
            let target_threshold = live_items * self.target_utilisation;
            self.gc_threshold = core::cmp::max(target_threshold, self.min_heap_size);

            // Shrink the `to_space` as much as possible
            self.to_space.shrink_to_fit();

            // Shrink the `from_space` as much as possible
            self.from_space.shrink_to_fit();
        }
    }

    /// Internal helper to migrate a single value descriptor if it's a pointer.
    fn migrate_value(&mut self, val: &mut Value) {
        // Extract the original heap space index if the value is an object or array
        let (Value::Object(old_idx) | Value::Array(old_idx)) = *val else {
            // We don't need to migrate the other non-heap primitives
            return;
        };

        // Ensure that its within bounds
        if old_idx >= self.from_space.len() {
            debug_assert!(false);
            return;
        }

        // This item was already migrated earlier in this GC cycle
        // Just update the current value pointer to the new location.
        if let HeapItem::Forwarded(new_idx) = &self.from_space[old_idx] {
            *val = match *val {
                Value::Object(_) => Value::Object(*new_idx),
                Value::Array(_) => Value::Array(*new_idx),
                _ => unreachable!(),
            };
        // This item hasn't been migrated into the `to_space` yet
        } else {
            let new_idx = self.to_space.len();

            // Strip the old item out of `from_space`, replacing it with a Forwarded record
            // so future references to this item will refer to the new index
            let old_item =
                core::mem::replace(&mut self.from_space[old_idx], HeapItem::Forwarded(new_idx));

            // Push to the end of our `to_space` to add it to the new heap
            // so it doesn't get killed
            self.to_space.push(old_item);

            // Update the original value pointer to point to the new location
            *val = match *val {
                Value::Object(_) => Value::Object(new_idx),
                Value::Array(_) => Value::Array(new_idx),
                _ => unreachable!(),
            };
        }
    }
}

impl HeapAllocator for CheneyAllocator {
    fn ensure_capacity(&mut self, roots: &mut [&mut Value]) {
        if self.from_space.len() >= self.gc_threshold {
            self.collect(roots);
        }
    }

    fn alloc_raw(&mut self, item: HeapItem) -> usize {
        let idx = self.from_space.len();
        self.from_space.push(item);
        idx
    }

    fn get_item(&self, ptr: usize) -> &HeapItem {
        &self.from_space[ptr]
    }

    fn get_item_mut(&mut self, ptr: usize) -> &mut HeapItem {
        &mut self.from_space[ptr]
    }
}

impl Default for CheneyAllocator {
    fn default() -> Self {
        Self::new(256, 75, 25, 2, 256)
    }
}
