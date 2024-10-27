## nftables-async

An async version of the helper to run nftables in the `nftables` crate. Simply add both `nftables-async` and `nftables` to your crate, then use the `nftables_async::apply_ruleset` or `nftables_async::get_current_ruleset` to perform manipulations. Everything is compatible with the sync helper, even the error types, the functions, however, return "true" async futures.

The I/O in the futures is backed by the Tokio `process` module which doesn't require the runtime, so the library is still runtime-agnostic.
