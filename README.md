# Ink native support

Shared support sources for generated Ink programs, independently pinned by the
Rust backend and execution runtime.

The package exports typed value/JSON primitives, owned temporary collections,
and native/Wasm state exports.
Emission logic stays in lowering packages. Wire contracts live in
[ink-abi](https://github.com/siliconjungle/ink-abi).

The reference evaluator remains in `ink-core`; this package contains no parser,
source evaluator, optimisation search or proof authority. These operational
implementations and their toolchains remain part of the trusted execution boundary.

`cargo test --locked` compiles the exported typed and owned support together and
checks exact float bits, integer boundaries, tagged values and owned collections.
Distribution conformance additionally compiles complete Rust applications and
checks their actions, snapshots and Wasm exports against the reference evaluator.

`INDEXED_TABLE_RUST_SOURCE` supplies optional contiguous row slots with an ordered key-to-slot index. Keys and public views stay ordered; swap removal repairs moved slots. These primitives are trusted implementations, independently tested against std BTreeMap and distribution histories.

## License

Project code and documentation are [MIT licensed](LICENSE). Third-party dependencies,
vendored projects and research data retain their own licenses and notices.

Generic binary pure-call support is separate from JSON text and state transaction
reports. Generated signatures bind argument/result schemas; `call_binary` uses the
ABI's bounded codec, and optional `lang_call_binary` returns raw result bytes.
Networking stays external. Recursive checkpoint schemas use the core's shared
cycle facts and bounded Bind/Ref expansion, including recursive records in state
and event payloads. These are operationally trusted support implementations.

`TYPED_SNAPSHOT_RUST_SOURCE` supplies the direct typed writer/reader for the fixed portable snapshot format. The Rust lowering spells finite schemas over ordinary native values. Readers authenticate the entire framing/digest before decoding; limits, canonical scalars, ordered keys/events and final payload checks retain the shared codec contract. Recursive/column schemas fall back to the shared codec. This support changes allocation and value conversion, without granting algorithm or rewrite authority.
