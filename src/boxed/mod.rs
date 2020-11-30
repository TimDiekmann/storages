mod raw;

use core::{alloc::Layout, mem::ManuallyDrop, ops::{Deref, DerefMut}, ptr};

use alloc::alloc::{handle_alloc_error, Global};

use crate::storage::{AllocatorStorage, Storage, UnmanagedStorage, ValueStorage};

pub use self::raw::*;

pub struct Box<T, S: Storage = AllocatorStorage<T, Global>> {
    raw: RawBox<T, S>,
    allocator: S::Allocator,
}

impl<T> Box<T> {
    pub fn new(value: T) -> Self {
        let mut storage = AllocatorStorage::new(Global)
            .unwrap_or_else(|_| handle_alloc_error(Layout::new::<T>()));
        storage.as_mut().write(value);
        unsafe { Self::from_storage(storage) }
    }
}

impl<S: ValueStorage<Allocator = ()>> Box<S::Item, S> {
    pub fn with_storage(value: S::Item, storage: S) -> Self {
        Self {
            raw: RawBox::with_storage(value, storage),
            allocator: (),
        }
    }
}

impl<S: Storage<Allocator = ()>> Box<S::Item, S> {
    pub unsafe fn from_storage(storage: S) -> Self {
        Self {
            raw: RawBox::from_storage(storage),
            allocator: (),
        }
    }
}

impl<S: UnmanagedStorage + ValueStorage> Box<S::Item, S> {
    pub fn with_unmanaged_storage(value: S::Item, storage: S, allocator: S::Allocator) -> Self {
        Self {
            raw: RawBox::with_storage(value, storage),
            allocator,
        }
    }
}

impl<S: UnmanagedStorage> Box<S::Item, S> {
    pub unsafe fn from_unmanaged_storage(storage: S, allocator: S::Allocator) -> Self {
        Self {
            raw: RawBox::from_storage(storage),
            allocator,
        }
    }

    pub fn alloc(b: &Self) -> &S::Allocator {
        &b.allocator
    }
}

impl<S: Storage> Box<S::Item, S> {
    pub fn into_raw_box(b: Self) -> RawBox<S::Item, S>{
        let this = ManuallyDrop::new(b);
        unsafe { ptr::read(&this.raw) }
    }

    pub fn into_storage(b: Self) -> S {
        let this = ManuallyDrop::new(b);
        unsafe { ptr::read(Self::storage(&this)) }
    }

    pub fn storage(b: &Self) -> &S {
        RawBox::storage(&b.raw)
    }

    pub fn storage_mut(b: &mut Self) -> &mut S {
        RawBox::storage_mut(&mut b.raw)
    }
}

impl<T, S: Storage> Drop for Box<T, S> {
    default fn drop(&mut self) {
        // `RawBox` will handle dropping and deallocation
    }
}

impl<T, S: UnmanagedStorage> Drop for Box<T, S> {
    fn drop(&mut self) {
        // `RawBox` holds an unmanaged storage
        unsafe { self.raw.free(&self.allocator) }
    }
}

impl<S: ValueStorage> Deref for Box<S::Item, S> {
    type Target = S::Item;
    fn deref(&self) -> &Self::Target {
        unsafe { Box::storage(self).as_ref().assume_init_ref() }
    }
}

impl<S: ValueStorage> DerefMut for Box<S::Item, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { Box::storage_mut(self).as_mut().assume_init_mut() }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::UnmanagedAllocatorStorage;

    use super::*;

    #[test]
    fn boxed() {
        let managed_box_1 = Box::new(10);

        let storage: AllocatorStorage<u32, Global> = AllocatorStorage::new(Global).unwrap();
        let managed_box_2 = Box::with_storage(10, storage);
        let managed_box_3 = Box::with_storage(10, [10]);
        assert_eq!(*managed_box_1, *managed_box_2);
        assert_eq!(*managed_box_2, *managed_box_3);
    }
}
