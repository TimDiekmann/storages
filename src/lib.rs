#![no_std]
#![cfg_attr(doc, feature(doc_cfg, external_doc))]
#![cfg_attr(doc, doc(include = "../README.md"))]
#![feature(
    allocator_api,
    specialization,
    slice_ptr_len,
    slice_ptr_get,
    nonnull_slice_from_raw_parts,
    int_bits_const,
    ptr_internals,
    maybe_uninit_extra,
    maybe_uninit_ref,
    min_const_generics,
)]
#![allow(incomplete_features)]

extern crate alloc;

pub mod boxed;
pub mod storage;
