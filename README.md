## nftables-async

An async version of the helper to run nftables in the `nftables` crate. Simply add both `nftables-async` and `nftables` to your crate, then use the `nftables_async::apply_ruleset` or `nftables_async::get_current_ruleset` to perform manipulations. Everything is compatible with the sync helper, even the error types, the functions, however, return "true" async futures.

To provide the asynchronous I/O, an implementation of the `Process` trait in the crate is needed. Two implementations are provided built-in behind feature gates:

1. `TokioProcess` using the Tokio stack, enabled via `tokio-process` feature
2. `AsyncProcess` using the `async-process` crate (Smol stack), enabled via `async-process` feature.

### Why not the async helpers in nftables >0.6?

`nftables 0.6.0` introduced `tokio` and `async-process` features that are mostly equivalent to this crate, however, these have some disadvantages that make `nftables-async` still relevant:

1. The support in `nftables` is not implemented via a trait (like `nftables_async::Process`), meaning third-party extensions for async platforms other than Tokio or the `async-*` stack are not easily possible.
2. The two features are mutually exclusive, making it impossible to compile an `nftables` that has both enabled. This breaks the use-case of `fcnet`, that needs both when enabling multiple runtime features, and is generally inconvenient.
