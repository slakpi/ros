use crate::arch;
use core::marker::PhantomData;
use core::mem;

/// The slab structure.
///
///     Page Start
///     +---------------------+
///     | Free List Pointer   |
///     | In-Use Count        |
///     +---------------------+
///     | Next Object Pointer |
///     |         ...         |
///     +---------------------+
///     | Next Object Pointer |
///     |         ...         |
///     +---------------------+
///     |                     |
///    ...  (More Objects)   ...
///     |                     |
///     +---------------------+
///     Page End
///
#[repr(C)]
struct Slab {
  next: usize,
}

pub struct SlabAllocator<T> {
  first: usize,
  _phantom: PhantomData<T>
}

impl<T> SlabAllocator<T> {
  pub fn new() -> Self {
    // The size of T must be less than the page size minus the size of the
    // next pointer.
    debug_assert!(mem::size_of::<T>() < arch::get_page_size() - mem::size_of::<usize>());

    SlabAllocator{
      first: 0,
      _phantom: PhantomData,
    }
  }

  pub fn alloc() -> Option<&'static T> {
    None
  }
}
