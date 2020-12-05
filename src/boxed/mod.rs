mod raw;

pub use self::raw::*;

use crate::buffer::{AllocatedBuffer, Buffer, UnmanagedBuffer};
use alloc::alloc::Global;
use core::{
    mem,
    ops::{CoerceUnsized, Deref, DerefMut},
    ptr,
};
use mem::ManuallyDrop;

pub struct Box<T, B = AllocatedBuffer<T>, D = <B as Buffer<T>>::ExternalData>
where
    T: ?Sized,
    B: Buffer<T, ExternalData = D>,
{
    raw: RawBox<T, B>,
    data: D,
}

/// Construction of boxed values with a buffer backed by the global allocator.
#[allow(clippy::use_self)]
impl<T> Box<T> {
    /// Allocates memory on the global heap and then places `value` into it.
    ///
    /// This doesn't actually allocate if `T` is zero-sized.
    ///
    /// # Examples
    ///
    /// ```
    /// use storages::boxed::Box;
    ///
    /// let five = Box::new(5);
    ///
    /// assert_eq!(*five, 5);
    /// ```
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            raw: RawBox::new(value),
            data: Global,
        }
    }

    /// Constructs a new box with uninitialized contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use storages::boxed::Box;
    ///
    /// let mut five = Box::<u32>::new_uninit();
    ///
    /// let five = unsafe {
    ///     // Deferred initialization:
    ///     five.as_mut_ptr().write(5);
    ///
    ///     five.assume_init()
    /// };
    ///
    /// assert_eq!(*five, 5);
    /// ```
    #[inline]
    pub fn new_uninit() -> Box<mem::MaybeUninit<T>, AllocatedBuffer<T>> {
        Box {
            raw: RawBox::new_uninit(),
            data: Global,
        }
    }

    /// Constructs a new box with uninitialized contents, with the memory being filled with `0`
    /// bytes.
    ///
    /// See [`MaybeUninit::zeroed`] for examples of correct and incorrect usage of this method.
    ///
    /// [`MaybeUninit::zeroed`]: core::mem::MaybeUninit::zeroed
    ///
    /// # Examples
    ///
    /// ```
    /// use storages::boxed::Box;
    ///
    /// let zero = Box::<u32>::new_zeroed();
    /// let zero = unsafe { zero.assume_init() };
    ///
    /// assert_eq!(*zero, 0);
    /// ```
    #[inline]
    pub fn new_zeroed() -> Box<mem::MaybeUninit<T>, AllocatedBuffer<T>> {
        Box {
            raw: RawBox::new_zeroed(),
            data: Global,
        }
    }
}

/// Construction of boxed slices with a buffer backed by the global allocator.
#[allow(clippy::use_self)]
impl<T> Box<[T]> {
    /// Constructs a boxed slice with uninitialized contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use storages::boxed::Box;
    ///
    /// let mut values = Box::<[u32]>::new_uninit_slice(3);
    ///
    /// let values = unsafe {
    ///     // Deferred initialization:
    ///     values[0].as_mut_ptr().write(1);
    ///     values[1].as_mut_ptr().write(2);
    ///     values[2].as_mut_ptr().write(3);
    ///
    ///     values.assume_init()
    /// };
    ///
    /// assert_eq!(&*values, [1, 2, 3])
    /// ```
    #[inline]
    pub fn new_uninit_slice(len: usize) -> Box<[mem::MaybeUninit<T>], AllocatedBuffer<[T]>> {
        Box {
            raw: RawBox::new_uninit_slice(len),
            data: Global,
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
    /// use storages::boxed::Box;
    ///
    /// let values = Box::<[u32]>::new_zeroed_slice(3);
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(&*values, [0, 0, 0])
    /// ```
    #[inline]
    pub fn new_zeroed_slice(len: usize) -> Box<[mem::MaybeUninit<T>], AllocatedBuffer<[T]>> {
        Box {
            raw: RawBox::new_zeroed_slice(len),
            data: Global,
        }
    }
}

/// Construction of boxed values in a provided buffer.
#[allow(clippy::use_self)]
impl<T, B, D> Box<T, B, D>
where
    B: Buffer<T, ExternalData = D>,
{
    /// Places the value in the provided buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::alloc::System;
    /// use storages::{boxed::Box, buffer::AllocatedBuffer};
    ///
    /// let buffer = AllocatedBuffer::new_in(&System)?;
    /// let five = Box::new_in(5, buffer, System);
    ///
    /// assert_eq!(*five, 5);
    /// # Ok::<(), core::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn new_in(value: T, buffer: B, data: D) -> Self {
        Self {
            raw: RawBox::new_in(value, buffer, &data),
            data,
        }
    }

    /// Constructs a new box with uninitialized contents in the provided buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api, maybe_uninit_extra)]
    ///
    /// use std::alloc::System;
    /// use storages::{boxed::Box, buffer::AllocatedBuffer};
    ///
    /// let buffer = AllocatedBuffer::new_in(&System)?;
    /// let mut five = Box::<u32, _>::new_uninit_in(buffer, System);
    ///
    /// let five = unsafe {
    ///     // Deferred initialization:
    ///     five.as_mut_ptr().write(5);
    ///
    ///     five.assume_init()
    /// };
    ///
    /// assert_eq!(*five, 5);
    /// # Ok::<(), core::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn new_uninit_in(buffer: B, data: D) -> Box<mem::MaybeUninit<T>, B, D>
    where
        B: Buffer<mem::MaybeUninit<T>, ExternalData = D>,
    {
        Box {
            raw: RawBox::new_uninit_in(buffer),
            data,
        }
    }
}

/// Construction of boxed slices in a provided buffer.
#[allow(clippy::use_self)]
impl<T, B, D> Box<[T], B, D>
where
    B: Buffer<[T], ExternalData = D>,
{
    /// Constructs a boxed with uninitialized contents in the provided buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::mem;
    /// use storages::boxed::Box;
    ///
    /// let buffer = [mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut values = Box::<[u32], _>::new_uninit_slice_in(buffer, ());
    ///
    /// let values = unsafe {
    ///     // Deferred initialization:
    ///     values[0].as_mut_ptr().write(1);
    ///     values[1].as_mut_ptr().write(2);
    ///     values[2].as_mut_ptr().write(3);
    ///
    ///     values.assume_init()
    /// };
    ///
    /// assert_eq!(*values, [1, 2, 3])
    /// ```
    #[inline]
    pub fn new_uninit_slice_in(buffer: B, data: D) -> Box<[mem::MaybeUninit<T>], B, D>
    where
        B: Buffer<[mem::MaybeUninit<T>], ExternalData = D>,
    {
        Box {
            raw: RawBox::new_uninit_slice_in(buffer),
            data,
        }
    }
}

#[allow(clippy::use_self)]
impl<T, B, D> Box<mem::MaybeUninit<T>, B, D>
where
    B: Buffer<T, ExternalData = D> + Buffer<mem::MaybeUninit<T>, ExternalData = D>,
{
    /// Converts to `Box<T, B>`.
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
    /// use storages::boxed::Box;
    ///
    /// let mut five = Box::<u32>::new_uninit();
    ///
    /// let five = unsafe {
    ///     // Deferred initialization:
    ///     five.as_mut_ptr().write(5);
    ///
    ///     five.assume_init()
    /// };
    /// assert_eq!(*five, 5);
    /// ```
    #[inline]
    pub unsafe fn assume_init(self) -> Box<T, B, D> {
        let this = ManuallyDrop::new(self);
        Box {
            raw: ptr::read(&this.raw).assume_init(),
            data: ptr::read(&this.data),
        }
    }
}

#[allow(clippy::use_self)]
impl<T, B, D> Box<[mem::MaybeUninit<T>], B, D>
where
    B: Buffer<[T], ExternalData = D> + Buffer<[mem::MaybeUninit<T>], ExternalData = D>,
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
    /// use storages::boxed::Box;
    ///
    /// let buffer = [mem::MaybeUninit::<u32>::uninit(); 3];
    /// let mut values = Box::new_uninit_slice_in(buffer, ());
    ///
    /// let values = unsafe {
    ///     // Deferred initialization:
    ///     values[0].as_mut_ptr().write(1);
    ///     values[1].as_mut_ptr().write(2);
    ///     values[2].as_mut_ptr().write(3);
    ///
    ///     values.assume_init()
    /// };
    ///
    /// assert_eq!(*values, [1, 2, 3])
    /// ```
    #[inline]
    pub unsafe fn assume_init(self) -> Box<[T], B, D> {
        let this = ManuallyDrop::new(self);
        Box {
            raw: ptr::read(&this.raw).assume_init(),
            data: ptr::read(&this.data),
        }
    }
}

#[doc(hidden)]
impl<T: ?Sized, S: Buffer<T, ExternalData = D>, D> Drop for Box<T, S, D> {
    default fn drop(&mut self) {
        // buffer is managed, no drop needed
    }
}

impl<T, S, D> Drop for Box<T, S, D>
where
    T: ?Sized,
    S: UnmanagedBuffer<T, ExternalData = D>,
{
    fn drop(&mut self) {
        unsafe { self.raw.buffer_mut().free_unchecked(&self.data) }
    }
}

impl<T, B, D> Deref for Box<T, B, D>
where
    T: ?Sized,
    B: Buffer<T, ExternalData = D>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.raw.as_ref(&self.data)
    }
}

impl<T, B, D> DerefMut for Box<T, B, D>
where
    T: ?Sized,
    B: Buffer<T, ExternalData = D>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.raw.as_mut(&self.data)
    }
}

impl<T, U, D, BT, BU> CoerceUnsized<Box<U, BU, D>> for Box<T, BT, D>
where
    T: ?Sized,
    U: ?Sized,
    BT: Buffer<T, ExternalData = D> + CoerceUnsized<BU>,
    BU: Buffer<U, ExternalData = D>,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new() {
        let five = Box::new(5);

        assert_eq!(*five, 5);
    }
}
