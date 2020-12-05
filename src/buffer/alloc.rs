use super::{Buffer, UnmanagedBuffer};
use alloc::alloc::Global;
use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::{PhantomData, Unsize},
    mem,
    ops::CoerceUnsized,
    ptr::NonNull,
    slice,
};

#[derive(Copy, Clone)]
enum Init {
    Unspecified,
    Zeroed,
}

pub struct AllocatedBuffer<T: ?Sized, A: ?Sized = Global> {
    ptr: NonNull<T>,
    _owned: PhantomData<T>,
    _marker: PhantomData<fn(*const A)>,
}

impl<T: ?Sized, A: ?Sized> AllocatedBuffer<T, A> {
    pub unsafe fn from_raw(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            _owned: PhantomData,
            _marker: PhantomData,
        }
    }
}

impl<T> AllocatedBuffer<T> {
    pub fn new() -> Result<Self, AllocError> {
        Self::new_in(&Global)
    }

    pub fn new_zeroed() -> Result<Self, AllocError> {
        Self::new_zeroed_in(&Global)
    }
}

impl<T, A: ?Sized + Allocator> AllocatedBuffer<T, A> {
    fn allocate_in(allocator: &A, init: Init) -> Result<Self, AllocError> {
        let layout = Layout::new::<T>();
        let ptr = match init {
            Init::Unspecified => allocator.allocate(layout)?,
            Init::Zeroed => allocator.allocate_zeroed(layout)?,
        };
        unsafe { Ok(Self::from_raw(ptr.as_non_null_ptr().cast())) }
    }

    pub fn new_in(allocator: &A) -> Result<Self, AllocError> {
        Self::allocate_in(allocator, Init::Unspecified)
    }

    pub fn new_zeroed_in(allocator: &A) -> Result<Self, AllocError> {
        Self::allocate_in(allocator, Init::Zeroed)
    }
}

impl<T, A: ?Sized + Allocator> AllocatedBuffer<[T], A> {
    fn capacity_from_bytes(bytes: usize) -> usize {
        debug_assert_ne!(mem::size_of::<T>(), 0);
        bytes / mem::size_of::<T>()
    }

    fn allocate_slice(allocator: &A, len: usize, init: Init) -> Result<Self, AllocError> {
        let ptr = if mem::size_of::<T>() == 0 {
            NonNull::slice_from_raw_parts(NonNull::dangling(), 0)
        } else {
            let layout = Layout::array::<T>(len).map_err(|_| AllocError)?;
            alloc_guard(layout.size()).map_err(|_| AllocError)?;
            let ptr = match init {
                Init::Unspecified => allocator.allocate(layout)?,
                Init::Zeroed => allocator.allocate_zeroed(layout)?,
            };

            NonNull::slice_from_raw_parts(
                ptr.as_non_null_ptr().cast(),
                Self::capacity_from_bytes(ptr.len()),
            )
        };
        unsafe { Ok(Self::from_raw(ptr)) }
    }

    pub fn new_slice(allocator: &A, len: usize) -> Result<Self, AllocError> {
        Self::allocate_slice(allocator, len, Init::Unspecified)
    }

    pub fn new_slice_zeroed(allocator: &A, len: usize) -> Result<Self, AllocError> {
        Self::allocate_slice(allocator, len, Init::Zeroed)
    }
}

impl<T: ?Sized, A: ?Sized + Allocator> Buffer<T> for AllocatedBuffer<T, A> {
    type ExternalData = A;

    fn as_ptr(&self, _data: &Self::ExternalData) -> *const T {
        self.ptr.as_ptr()
    }

    fn as_mut_ptr(&mut self, _data: &Self::ExternalData) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T, A: ?Sized + Allocator> Buffer<mem::MaybeUninit<T>> for AllocatedBuffer<T, A> {
    type ExternalData = A;

    fn as_ptr(&self, _data: &Self::ExternalData) -> *const mem::MaybeUninit<T> {
        self.ptr.as_ptr().cast()
    }

    fn as_mut_ptr(&mut self, _data: &Self::ExternalData) -> *mut mem::MaybeUninit<T> {
        self.ptr.as_ptr().cast()
    }
}

impl<T, A: ?Sized + Allocator> Buffer<[mem::MaybeUninit<T>]> for AllocatedBuffer<[T], A> {
    type ExternalData = A;

    fn as_ptr(&self, _data: &Self::ExternalData) -> *const [mem::MaybeUninit<T>] {
        unsafe { slice::from_raw_parts(self.ptr.cast().as_ptr(), self.ptr.len()) }
    }

    fn as_mut_ptr(&mut self, _data: &Self::ExternalData) -> *mut [mem::MaybeUninit<T>] {
        unsafe { slice::from_raw_parts_mut(self.ptr.cast().as_ptr(), self.ptr.len()) }
    }
}

impl<T: ?Sized, A: Allocator> UnmanagedBuffer<T> for AllocatedBuffer<T, A> {
    unsafe fn free_unchecked(&mut self, allocator: &Self::ExternalData) {
        let size = mem::size_of_val(self.ptr.as_ref());
        let align = mem::align_of_val(self.ptr.as_ref());
        let layout = Layout::from_size_align_unchecked(size, align);
        allocator.deallocate(self.ptr.cast(), layout);
    }
}

#[inline]
const fn alloc_guard(alloc_size: usize) -> Result<(), AllocError> {
    if usize::BITS < 64 && alloc_size > isize::MAX as usize {
        Err(AllocError)
    } else {
        Ok(())
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized, A: Allocator> CoerceUnsized<AllocatedBuffer<U, A>>
    for AllocatedBuffer<T, A>
{
}
