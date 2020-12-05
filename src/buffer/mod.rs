mod alloc;
mod array;

pub use self::alloc::*;

/// Backend for collection types like `Box` and `Vec`.
///
/// Every buffer may require an external datum, which is passed every time the buffer is accessed.
/// This way the implementation does not require the data to store which might save some memory.
pub trait Buffer<T: ?Sized> {
    /// Data required to be passed at every interaction with the buffer.
    type ExternalData: ?Sized;

    /// Returns a shared pointer to the buffered datum.
    fn as_ptr(&self, data: &Self::ExternalData) -> *const T;

    /// Returns a unique pointer to the buffered datum.
    fn as_mut_ptr(&mut self, data: &Self::ExternalData) -> *mut T;
}

/// A buffer, which uses an external resource
pub trait UnmanagedBuffer<T: ?Sized>: Buffer<T> {
    /// Frees the backed resource.
    ///
    /// # Safety
    ///
    /// The buffer must not be used after calling this method
    unsafe fn free_unchecked(&mut self, allocator: &Self::ExternalData);

    /// Frees the backed resource
    fn free(mut self, allocator: &Self::ExternalData)
    where
        Self: Sized,
    {
        unsafe {
            self.free_unchecked(allocator);
        }
        drop(self)
    }
}
