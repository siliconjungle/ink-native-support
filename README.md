# Ink native support

Shared support sources for generated Ink programs, independently pinned by the
Rust backend and execution runtime.

The package exports typed value/JSON primitives, owned temporary collections,
the dynamic value runtime used by emitted C, and native/Wasm state exports.
Emission logic stays in lowering packages. Wire contracts live in
[ink-abi](https://github.com/siliconjungle/ink-abi).

Typed generated values and dynamic handles retain their different representations.
The reference evaluator remains in `ink-core`; this package contains no parser,
source evaluator, optimisation search or proof authority. These operational
implementations and their toolchains remain part of the trusted execution boundary.

`cargo test --locked` compiles the exported typed and owned support together and
checks exact float bits, integer boundaries, tagged values and owned collections.
Distribution conformance additionally compiles complete C/Rust applications and
checks their actions, snapshots and Wasm exports against the reference evaluator.
