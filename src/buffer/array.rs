use core::{mem, ptr};

use super::Buffer;

impl<T, const N: usize> Buffer<[T]> for [T; N] {
    type ExternalData = ();

    fn as_ptr(&self, _data: &Self::ExternalData) -> *const [T] {
        ptr::slice_from_raw_parts(<[_]>::as_ptr(self).cast(), N)
    }

    fn as_mut_ptr(&mut self, _data: &Self::ExternalData) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(<[_]>::as_mut_ptr(self).cast(), N)
    }
}

impl<T, const N: usize> Buffer<[T]> for [mem::MaybeUninit<T>; N] {
    type ExternalData = ();

    fn as_ptr(&self, _data: &Self::ExternalData) -> *const [T] {
        ptr::slice_from_raw_parts(<[_]>::as_ptr(self).cast(), N)
    }

    fn as_mut_ptr(&mut self, _data: &Self::ExternalData) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(<[_]>::as_mut_ptr(self).cast(), N)
    }
}
