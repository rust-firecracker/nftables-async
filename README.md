## nftables-async

An async version of the helper to run nftables in the `nftables` crate. Simply add both `nftables-async` and `nftables` to your crate, then use the `nftables_async::apply_ruleset` or `nftables_async::get_current_ruleset` to perform manipulations. Everything is compatible with the sync helper, even the error types, the functions, however, return "true" async futures.

To provide the asynchronous I/O, an implementation of the `Process` trait in the crate is needed. Two implementations are provided built-in behind feature gates:

1. `TokioProcess` using the Tokio stack, enabled via `tokio-process` feature
2. `AsyncProcess` using the `async-process` crate (Smol stack), enabled via `async-process` feature
