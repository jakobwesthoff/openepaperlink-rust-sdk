# Implement FusedStream for EventStream

`EventStream` implements `Stream` but not `FusedStream`. Callers using
`tokio::select!` with the stream need to call `.fuse()` manually, or they
risk unexpected polls after the stream terminates. Adding a `FusedStream`
impl (with an internal terminated flag) would make reconnection loops more
ergonomic.
