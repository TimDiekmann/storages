[![Test Status](https://github.com/TimDiekmann/storages/workflows/Test/badge.svg?event=push&branch=main)](https://github.com/TimDiekmann/storages/actions?query=workflow%3ATest+event%3Apush+branch%3Amain)
[![Coverage Status](https://codecov.io/gh/TimDiekmann/storages/branch/main/graph/badge.svg)](https://codecov.io/gh/TimDiekmann/storages)
[![Docs main](https://img.shields.io/static/v1?label=docs&message=main&color=5479ab)](https://timdiekmann.github.io/storages/storages/index.html)
[![Docs.rs](https://docs.rs/storages/badge.svg)](https://docs.rs/storages)
[![Crates.io](https://img.shields.io/crates/v/storages)](https://crates.io/crates/storages)
[![Crates.io](https://img.shields.io/crates/l/storages)](#license)

Test environment for storage-based collection rather than allocator-based.

As this crate is designed for replacing allocators in collections if desired,
it requires a nightly compiler.

Contributing
------------

I'm currently changing pretty much everything to test different approaches.
While pull requests are always welcome, it may be better to open an issue
and make suggestions or post it at the [topic on internals.rust-lang.org](
https://internals.rust-lang.org/t/is-custom-allocators-the-right-abstraction/13460). 

License
-------

`storages` is distributed under the terms of both the MIT license and the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/TimDiekmann/storages/blob/main/LICENSE-APACHE) and [LICENSE-MIT](https://github.com/TimDiekmann/storages/blob/main/LICENSE-MIT) for details.
