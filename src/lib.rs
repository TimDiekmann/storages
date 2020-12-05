#![no_std]
#![cfg_attr(doc, feature(doc_cfg, external_doc))]
#![cfg_attr(doc, doc(include = "../README.md"))]
// required features
#![feature(
    allocator_api,
    specialization,
    coerce_unsized,
    unsize,
    min_const_generics
)]
// convenient features
#![feature(
    nonnull_slice_from_raw_parts,
    int_bits_const,
    slice_ptr_len,
    slice_ptr_get
)]
#![allow(incomplete_features)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_safety_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]
#![doc(test(attr(
    deny(
        future_incompatible,
        macro_use_extern_crate,
        nonstandard_style,
        rust_2018_compatibility,
        rust_2018_idioms,
        trivial_casts,
        trivial_numeric_casts,
        unused,
        unused_import_braces,
        unused_lifetimes,
        unused_qualifications,
        variant_size_differences,
    ),
    allow(unused_extern_crates)
)))]
#![warn(
    future_incompatible,
    macro_use_extern_crate,
    nonstandard_style,
    rust_2018_compatibility,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]

extern crate alloc;

pub mod boxed;
pub mod buffer;
