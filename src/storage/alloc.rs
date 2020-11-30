use super::{ContiguousStorage, Storage, UnmanagedStorage, ValueStorage};
use core::{
    alloc::{AllocError, AllocRef, Layout},
    marker::PhantomData,
    mem,
    ptr::{NonNull, Unique},
    slice,
};

enum Init {
    Unspecified,
    Zeroed,
}

pub struct UnmanagedAllocatorStorage<T: ?Sized, A: AllocRef>(Unique<T>, PhantomData<*const A>);

pub struct AllocatorStorage<T: ?Sized, A: AllocRef>(UnmanagedAllocatorStorage<T, A>, A);

impl<T, A: AllocRef> UnmanagedAllocatorStorage<T, A> {
    fn allocate(allocator: &A, init: Init) -> Result<Self, AllocError> {
        let layout = Layout::new::<T>();
        let ptr = match init {
            Init::Unspecified => allocator.alloc(layout)?,
            Init::Zeroed => allocator.alloc_zeroed(layout)?,
        };
        let unique = unsafe { Unique::new_unchecked(ptr.as_ptr()) };
        Ok(Self(unique.cast(), PhantomData))
    }

    pub fn new(allocator: &A) -> Result<Self, AllocError> {
        Self::allocate(allocator, Init::Unspecified)
    }

    pub fn new_zeroed(allocator: &A) -> Result<Self, AllocError> {
        Self::allocate(allocator, Init::Zeroed)
    }
}

impl<T, A: AllocRef> AllocatorStorage<T, A> {
    pub fn new(allocator: A) -> Result<Self, AllocError> {
        let storage = UnmanagedAllocatorStorage::new(&allocator)?;
        Ok(Self(storage, allocator))
    }

    pub fn new_zeroed(allocator: A) -> Result<Self, AllocError> {
        let storage = UnmanagedAllocatorStorage::new_zeroed(&allocator)?;
        Ok(Self(storage, allocator))
    }
}

impl<T, A: AllocRef> UnmanagedAllocatorStorage<[T], A> {
    fn capacity_from_bytes(bytes: usize) -> usize {
        debug_assert_ne!(mem::size_of::<T>(), 0);
        bytes / mem::size_of::<T>()
    }

    fn allocate(allocator: &A, len: usize, init: Init) -> Result<Self, AllocError> {
        if mem::size_of::<T>() == 0 {
            let ptr = NonNull::slice_from_raw_parts(NonNull::dangling(), 0);
            Ok(unsafe { Self(Unique::new_unchecked(ptr.as_ptr()), PhantomData) })
        } else {
            let layout = Layout::array::<T>(len).map_err(|_| AllocError)?;
            alloc_guard(layout.size()).map_err(|_| AllocError)?;
            let ptr = match init {
                Init::Unspecified => allocator.alloc(layout)?,
                Init::Zeroed => allocator.alloc_zeroed(layout)?,
            };

            let ptr = NonNull::slice_from_raw_parts(
                ptr.as_non_null_ptr().cast(),
                Self::capacity_from_bytes(ptr.len()),
            );
            Ok(unsafe { Self(Unique::new_unchecked(ptr.as_ptr()), PhantomData) })
        }
    }

    pub fn new_slice(allocator: &A, len: usize) -> Result<Self, AllocError> {
        Self::allocate(allocator, len, Init::Unspecified)
    }

    pub fn new_slice_zeroed(allocator: &A, len: usize) -> Result<Self, AllocError> {
        Self::allocate(allocator, len, Init::Zeroed)
    }
}

impl<T, A: AllocRef> AllocatorStorage<[T], A> {
    pub fn new_slice(allocator: A, len: usize) -> Result<Self, AllocError> {
        let storage = UnmanagedAllocatorStorage::new_slice(&allocator, len)?;
        Ok(Self(storage, allocator))
    }

    pub fn new_slice_zeroed(allocator: A, len: usize) -> Result<Self, AllocError> {
        let storage = UnmanagedAllocatorStorage::new_slice_zeroed(&allocator, len)?;
        Ok(Self(storage, allocator))
    }
}

impl<T, A: AllocRef> Storage for UnmanagedAllocatorStorage<T, A> {
    type Allocator = A;
    type Item = T;
}

impl<T, A: AllocRef> Storage for UnmanagedAllocatorStorage<[T], A> {
    type Allocator = A;
    type Item = T;
}

impl<T, A: AllocRef> Storage for AllocatorStorage<T, A> {
    type Allocator = ();
    type Item = T;
}

impl<T, A: AllocRef> Storage for AllocatorStorage<[T], A> {
    type Allocator = ();
    type Item = T;
}

impl<T, A: AllocRef> UnmanagedStorage for UnmanagedAllocatorStorage<T, A> {
    unsafe fn free(&mut self, allocator: &Self::Allocator) {
        allocator.dealloc(self.0.cast().into(), Layout::new::<T>());
    }
}

impl<T, A: AllocRef> UnmanagedStorage for UnmanagedAllocatorStorage<[T], A> {
    unsafe fn free(&mut self, allocator: &Self::Allocator) {
        let ptr = NonNull::from(self.0);
        if mem::size_of::<T>() != 0 {
            let layout = Layout::from_size_align_unchecked(
                mem::size_of::<T>() * ptr.len(),
                mem::align_of::<T>(),
            );
            allocator.dealloc(ptr.as_non_null_ptr().cast(), layout)
        }
    }
}

#[doc(hidden)]
impl<T: ?Sized, A: AllocRef> Drop for AllocatorStorage<T, A> {
    default fn drop(&mut self) {
        unreachable!()
    }
}

impl<T, A: AllocRef> Drop for AllocatorStorage<T, A> {
    fn drop(&mut self) {
        unsafe { self.0.free(&self.1) }
    }
}

impl<T, A: AllocRef> Drop for AllocatorStorage<[T], A> {
    fn drop(&mut self) {
        unsafe { self.0.free(&self.1) }
    }
}

impl<T, A: AllocRef> ValueStorage for UnmanagedAllocatorStorage<T, A> {
    fn as_ref(&self) -> &mem::MaybeUninit<Self::Item> {
        unsafe { &*self.0.cast().as_ptr() }
    }

    fn as_mut(&mut self) -> &mut mem::MaybeUninit<Self::Item> {
        unsafe { &mut *self.0.cast().as_ptr() }
    }
}

impl<T, A: AllocRef> ValueStorage for AllocatorStorage<T, A> {
    fn as_ref(&self) -> &mem::MaybeUninit<Self::Item> {
        self.0.as_ref()
    }

    fn as_mut(&mut self) -> &mut mem::MaybeUninit<Self::Item> {
        self.0.as_mut()
    }
}

impl<T, A: AllocRef> ContiguousStorage for UnmanagedAllocatorStorage<[T], A> {
    fn as_slice(&self) -> &[mem::MaybeUninit<Self::Item>] {
        let ptr = NonNull::from(self.0);
        unsafe { slice::from_raw_parts(ptr.cast().as_ptr(), ptr.len()) }
    }

    fn as_slice_mut(&mut self) -> &mut [mem::MaybeUninit<Self::Item>] {
        let ptr = NonNull::from(self.0);
        unsafe { slice::from_raw_parts_mut(ptr.cast().as_ptr(), ptr.len()) }
    }
}

impl<T, A: AllocRef> ContiguousStorage for AllocatorStorage<[T], A> {
    fn as_slice(&self) -> &[mem::MaybeUninit<Self::Item>] {
        self.0.as_slice()
    }

    fn as_slice_mut(&mut self) -> &mut [mem::MaybeUninit<Self::Item>] {
        self.0.as_slice_mut()
    }
}

#[inline]
fn alloc_guard(alloc_size: usize) -> Result<(), AllocError> {
    if usize::BITS < 64 && alloc_size > isize::MAX as usize {
        Err(AllocError)
    } else {
        Ok(())
    }
}
