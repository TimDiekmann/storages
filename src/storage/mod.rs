use core::mem;

mod alloc;
mod array;

pub use self::alloc::*;

pub trait Storage {
    type Allocator;
    type Item;
}

pub trait UnmanagedStorage: Storage {
    unsafe fn free(&mut self, allocator: &Self::Allocator);
}

pub trait ValueStorage: Storage {
    fn as_ref(&self) -> &mem::MaybeUninit<Self::Item>;

    fn as_mut(&mut self) -> &mut mem::MaybeUninit<Self::Item>;
}

pub trait ContiguousStorage: Storage {
    fn as_slice(&self) -> &[mem::MaybeUninit<Self::Item>];

    fn as_slice_mut(&mut self) -> &mut [mem::MaybeUninit<Self::Item>];
}
