use core::mem;

use super::{Storage, ValueStorage};

impl<T, const N: usize> Storage for [T; N] {
    type Allocator = ();
    type Item = T;
}

impl<T, const N: usize> ValueStorage for [T; N] {
    fn as_ref(&self) -> &mem::MaybeUninit<Self::Item> {
        unsafe { &*self.as_ptr().cast() }
    }

    fn as_mut(&mut self) -> &mut mem::MaybeUninit<Self::Item> {
        unsafe { &mut *self.as_mut_ptr().cast() }
    }
}
