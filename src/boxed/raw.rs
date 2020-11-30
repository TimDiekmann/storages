use core::{marker::PhantomData, mem::ManuallyDrop, ptr};

use crate::storage::{Storage, UnmanagedStorage, ValueStorage};

pub struct RawBox<T, S: Storage> {
    storage: S,
    _marker: PhantomData<*const T>,
}

impl<S: ValueStorage> RawBox<S::Item, S> {
    pub fn with_storage(value: S::Item, mut storage: S) -> Self {
        storage.as_mut().write(value);
        Self {
            storage,
            _marker: PhantomData,
        }
    }
}

impl<S: Storage> RawBox<S::Item, S> {
    pub unsafe fn from_storage(storage: S) -> Self {
        Self {
            storage,
            _marker: PhantomData,
        }
    }

    pub fn storage(b: &Self) -> &S {
        &b.storage
    }

    pub fn storage_mut(b: &mut Self) -> &mut S {
        &mut b.storage
    }

    pub fn into_storage(b: Self) -> S {
        let this = ManuallyDrop::new(b);
        unsafe { ptr::read(&this.storage) }
    }
}

impl<T, S: UnmanagedStorage> RawBox<T, S> {
    pub unsafe fn free(&mut self, allocator: &S::Allocator) {
        self.storage.free(allocator)
    }
}
