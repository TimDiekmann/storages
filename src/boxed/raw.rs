use core::{
    alloc::Layout,
    marker::PhantomData,
    mem::{self},
    ops::CoerceUnsized,
};

use alloc::alloc::{handle_alloc_error, Global};

use crate::buffer::{AllocatedBuffer, Buffer, UnmanagedBuffer};

/// A thin wrapper around a buffer.
///
/// `RawBox<T, B>` abstracts over a [`Buffer`], which holds values of `T`. As a buffer may need
/// external data, every operation on `RawBox`, which communicates with the underlying buffer,
/// requires the external data to be passed. For an [`AllocatedBuffer<T, A>`] this means, that
/// `A` has to be passed as a reference. When [`B::ExternalData`] is a zero-sized type, there
/// is no reason not to use [`Box`] directly instead.
///
/// Note, that `RawBox` does **not** implement [`Drop`] so it must be cleaned up after usage.
///
/// When storing the data next to the storage is not an issue, [`Box`] should be used instead.
///
/// [`Box`]: crate::boxed::Box
/// [`B::ExternalData`]: crate::buffer::Buffer::ExternalData
///
/// # Examples
///
/// Allocating a `RawBox` in the system allocator:
///
/// ```
/// #![feature(allocator_api)]
///
/// use std::alloc::System;
/// use storages::{boxed::RawBox, buffer::AllocatedBuffer};
///
/// let buffer = AllocatedBuffer::new_in(&System)?;
/// # #[allow(unused_variables)]
/// let five = RawBox::new_in(5, buffer, &System);
///
/// assert_eq!(*five.as_ref(&System), 5);
///
/// five.free(&System);
/// # Ok::<(), core::alloc::AllocError>(())
/// ```
///
/// When using a buffer, which doesn't require external data, the usage is similar to [`Box`]:
///
/// ```
/// use std::mem;
/// use storages::boxed::RawBox;
///
/// let buffer = [mem::MaybeUninit::<u32>::uninit(); 3];
/// let mut values = RawBox::new_uninit_slice_in(buffer);
///
/// let values = unsafe {
///     // Deferred initialization:
///     values.as_mut(&())[0].as_mut_ptr().write(1);
///     values.as_mut(&())[1].as_mut_ptr().write(2);
///     values.as_mut(&())[2].as_mut_ptr().write(3);
///
///     values.assume_init()
/// };
///
/// assert_eq!(*values.as_ref(&()), [1, 2, 3]);
/// ```
pub struct RawBox<T, B = AllocatedBuffer<T>>
where
    T: ?Sized,
    B: Buffer<T> + ?Sized,
{
    _marker: PhantomData<fn() -> *const T>,
    buffer: B,
}

/// Construction of boxed values with a buffer backed by the global allocator.
#[allow(clippy::use_self)]
impl<T> RawBox<T> {
    fn global_allocator_storage() -> AllocatedBuffer<T> {
        AllocatedBuffer::new().unwrap_or_else(|_| handle_alloc_error(Layout::new::<T>()))
    }

    /// Allocates memory on the global heap and then places `value` into it.
    ///
    /// This doesn't actually allocate if `T` is zero-sized.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::alloc::Global;
    /// use storages::boxed::RawBox;
    ///
    /// let five = RawBox::new(5);
    ///
    /// assert_eq!(*five.as_ref(&Global), 5);
    ///
    /// five.free(&Global);
    /// ```
    #[inline]
    pub fn new(value: T) -> Self {
        Self::new_in(value, Self::global_allocator_storage(), &Global)
    }

    /// Constructs a new raw box with uninitialized contents.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api, new_uninit)]
    ///
    /// use std::alloc::Global;
    /// use storages::boxed::RawBox;
    ///
    /// let mut five = RawBox::<u32>::new_uninit();
    ///
    /// let five = unsafe {
    ///     // Deferred initialization:
    ///     five.as_mut(&Global).as_mut_ptr().write(5);
    ///
    ///     five.assume_init()
    /// };
    ///
    /// assert_eq!(*five.as_ref(&Global), 5);
    ///
    /// five.free(&Global);
    /// ```
    #[inline]
    pub fn new_uninit() -> RawBox<mem::MaybeUninit<T>, AllocatedBuffer<T>> {
        Self::new_uninit_in(Self::global_allocator_storage())
    }

    /// Constructs a new raw box with uninitialized contents, with the memory being filled with `0`
    /// bytes.
    ///
    /// See [`MaybeUninit::zeroed`] for examples of correct and incorrect usage of this method.
    ///
    /// [`MaybeUninit::zeroed`]: core::mem::MaybeUninit::zeroed
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api, new_uninit)]
    ///
    /// use std::alloc::Global;
    /// use storages::boxed::RawBox;
    ///
    /// let zero = RawBox::<u32>::new_zeroed();
    /// let zero = unsafe { zero.assume_init() };
    ///
    /// assert_eq!(*zero.as_ref(&Global), 0);
    ///
    /// zero.free(&Global);
    /// ```
    #[inline]
    pub fn new_zeroed() -> RawBox<mem::MaybeUninit<T>, AllocatedBuffer<T>> {
        Self::new_uninit_in(
            AllocatedBuffer::new_zeroed()
                .unwrap_or_else(|_| handle_alloc_error(Layout::new::<T>())),
        )
    }
}

/// Construction of boxed slices with a buffer backed by the global allocator.
#[allow(clippy::use_self)]
impl<T> RawBox<[T]> {
    /// Constructs a boxed slice with uninitialized contents.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::alloc::Global;
    /// use storages::boxed::RawBox;
    ///
    /// let mut values = RawBox::<[u32]>::new_uninit_slice(3);
    ///
    /// let values = unsafe {
    ///     // Deferred initialization:
    ///     values.as_mut(&Global)[0].as_mut_ptr().write(1);
    ///     values.as_mut(&Global)[1].as_mut_ptr().write(2);
    ///     values.as_mut(&Global)[2].as_mut_ptr().write(3);
    ///
    ///     values.assume_init()
    /// };
    ///
    /// assert_eq!(*values.as_ref(&Global), [1, 2, 3]);
    ///
    /// values.free(&Global);
    /// ```
    #[inline]
    pub fn new_uninit_slice(len: usize) -> RawBox<[mem::MaybeUninit<T>], AllocatedBuffer<[T]>> {
        RawBox {
            buffer: AllocatedBuffer::new_slice(&Global, len)
                .unwrap_or_else(|_| handle_alloc_error(Layout::array::<T>(len).unwrap())),
            _marker: PhantomData,
        }
    }

    /// Constructs a boxed with uninitialized contents with the memory being filled with `0` bytes.
    ///
    /// See [`MaybeUninit::zeroed`] for examples of correct and incorrect usage of this method.
    ///
    /// [`MaybeUninit::zeroed`]: core::mem::MaybeUninit::zeroed
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::alloc::Global;
    /// use storages::boxed::RawBox;
    ///
    /// let values = RawBox::<[u32]>::new_zeroed_slice(3);
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(*values.as_ref(&Global), [0, 0, 0]);
    ///
    /// values.free(&Global);
    /// ```
    #[inline]
    pub fn new_zeroed_slice(len: usize) -> RawBox<[mem::MaybeUninit<T>], AllocatedBuffer<[T]>> {
        RawBox {
            buffer: AllocatedBuffer::new_slice_zeroed(&Global, len)
                .unwrap_or_else(|_| handle_alloc_error(Layout::array::<T>(len).unwrap())),
            _marker: PhantomData,
        }
    }
}

/// Construction of boxed values in a provided buffer.
#[allow(clippy::use_self)]
impl<T, B> RawBox<T, B>
where
    B: Buffer<T>,
{
    /// Places the value in the provided buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::alloc::System;
    /// use storages::{boxed::RawBox, buffer::AllocatedBuffer};
    ///
    /// let buffer = AllocatedBuffer::new_in(&System)?;
    /// let five = RawBox::new_in(5, buffer, &System);
    ///
    /// assert_eq!(*five.as_ref(&System), 5);
    ///
    /// five.free(&System);
    /// # Ok::<(), core::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn new_in(value: T, mut buffer: B, data: &B::ExternalData) -> Self {
        unsafe { buffer.as_mut_ptr(data).write(value) };
        Self {
            buffer,
            _marker: PhantomData,
        }
    }

    /// Constructs a new raw box with uninitialized contents in the provided buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::alloc::System;
    /// use storages::{boxed::RawBox, buffer::AllocatedBuffer};
    ///
    /// let buffer = AllocatedBuffer::new_in(&System)?;
    /// let mut five = RawBox::<u32, _>::new_uninit_in(buffer);
    ///
    /// let five = unsafe {
    ///     // Deferred initialization:
    ///     five.as_mut(&System).as_mut_ptr().write(5);
    ///
    ///     five.assume_init()
    /// };
    ///
    /// assert_eq!(*five.as_ref(&System), 5);
    ///
    /// five.free(&System);
    /// # Ok::<(), core::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn new_uninit_in(buffer: B) -> RawBox<mem::MaybeUninit<T>, B>
    where
        B: Buffer<mem::MaybeUninit<T>>,
    {
        RawBox {
            buffer,
            _marker: PhantomData,
        }
    }
}

/// Construction of boxed slices in a provided buffer.
#[allow(clippy::use_self)]
impl<T, B> RawBox<[T], B>
where
    B: Buffer<[T]>,
{
    /// Constructs a boxed with uninitialized contents in the provided buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::mem;
    /// use storages::boxed::RawBox;
    ///
    /// let buffer = [mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut values = RawBox::new_uninit_slice_in(buffer);
    ///
    /// let values = unsafe {
    ///     // Deferred initialization:
    ///     values.as_mut(&())[0].as_mut_ptr().write(1);
    ///     values.as_mut(&())[1].as_mut_ptr().write(2);
    ///     values.as_mut(&())[2].as_mut_ptr().write(3);
    ///
    ///     values.assume_init()
    /// };
    ///
    /// assert_eq!(*values.as_ref(&()), [1, 2, 3]);
    /// ```
    #[inline]
    pub fn new_uninit_slice_in(buffer: B) -> RawBox<[mem::MaybeUninit<T>], B>
    where
        B: Buffer<[mem::MaybeUninit<T>]>,
    {
        RawBox {
            buffer,
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized, B> RawBox<T, B>
where
    B: Buffer<T>,
{
    /// Creates a raw box from the provided buffer.
    ///
    /// # Safety
    ///
    /// It is up to the caller to guarantee that the value really is in an initialized state.
    /// Calling this when the content is not yet fully initialized causes immediate undefined
    /// behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use storages::boxed::RawBox;
    ///
    /// let values = unsafe { RawBox::from_buffer([0_u32; 3]) };
    ///
    /// assert_eq!(*values.as_ref(&()), [0, 0, 0]);
    /// ```
    pub unsafe fn from_buffer(buffer: B) -> Self {
        Self {
            buffer,
            _marker: PhantomData,
        }
    }
}

#[allow(clippy::use_self)]
impl<T, B> RawBox<mem::MaybeUninit<T>, B>
where
    B: Buffer<T> + Buffer<mem::MaybeUninit<T>>,
{
    /// Converts to `RawBox<T, B>`.
    ///
    /// # Safety
    ///
    /// As with [`MaybeUninit::assume_init`], it is up to the caller to guarantee that the value
    /// really is in an initialized state. Calling this when the content is not yet fully
    /// initialized causes immediate undefined behavior.
    ///
    /// [`MaybeUninit::assume_init`]: core::mem::MaybeUninit::assume_init
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api, new_uninit)]
    ///
    /// use std::alloc::Global;
    /// use storages::boxed::RawBox;
    ///
    /// let mut five = RawBox::<u32>::new_uninit();
    ///
    /// let five = unsafe {
    ///     // Deferred initialization:
    ///     five.as_mut(&Global).as_mut_ptr().write(5);
    ///
    ///     five.assume_init()
    /// };
    ///
    /// assert_eq!(*five.as_ref(&Global), 5);
    ///
    /// five.free(&Global);
    /// ```
    #[inline]
    pub unsafe fn assume_init(self) -> RawBox<T, B> {
        RawBox {
            buffer: self.buffer,
            _marker: PhantomData,
        }
    }
}

#[allow(clippy::use_self)]
impl<T, B> RawBox<[mem::MaybeUninit<T>], B>
where
    B: Buffer<[T]> + Buffer<[mem::MaybeUninit<T>]>,
{
    /// Constructs a boxed with uninitialized contents in the provided buffer.
    ///
    /// # Safety
    ///
    /// As with [`MaybeUninit::assume_init`], it is up to the caller to guarantee that the value
    /// really is in an initialized state. Calling this when the content is not yet fully
    /// initialized causes immediate undefined behavior.
    ///
    /// [`MaybeUninit::assume_init`]: core::mem::MaybeUninit::assume_init
    ///
    /// # Examples
    ///
    /// ```
    /// use std::mem;
    /// use storages::boxed::RawBox;
    ///
    /// let buffer = [mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut values = RawBox::new_uninit_slice_in(buffer);
    ///
    /// let values = unsafe {
    ///     // Deferred initialization:
    ///     values.as_mut(&())[0].as_mut_ptr().write(1);
    ///     values.as_mut(&())[1].as_mut_ptr().write(2);
    ///     values.as_mut(&())[2].as_mut_ptr().write(3);
    ///
    ///     values.assume_init()
    /// };
    ///
    /// assert_eq!(*values.as_ref(&()), [1, 2, 3]);
    /// ```
    #[inline]
    pub unsafe fn assume_init(self) -> RawBox<[T], B> {
        RawBox {
            buffer: self.buffer,
            _marker: PhantomData,
        }
    }
}

impl<T, B> RawBox<T, B>
where
    T: ?Sized,
    B: Buffer<T>,
{
    pub fn free(self, data: &B::ExternalData)
    where
        B: UnmanagedBuffer<T>,
    {
        self.buffer.free(data)
    }

    pub fn buffer(&self) -> &B {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut B {
        &mut self.buffer
    }

    pub fn as_ref(&self, data: &B::ExternalData) -> &T {
        unsafe { &*self.buffer.as_ptr(data) }
    }

    pub fn as_mut(&mut self, data: &B::ExternalData) -> &mut T {
        unsafe { &mut *self.buffer.as_mut_ptr(data) }
    }
}

impl<T, U, BT, BU> CoerceUnsized<RawBox<U, BU>> for RawBox<T, BT>
where
    T: ?Sized,
    U: ?Sized,
    BT: Buffer<T> + CoerceUnsized<BU>,
    BU: Buffer<U>,
{
}
