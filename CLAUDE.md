# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

RRedis is an in-memory key-value database with a custom protobuf-based network protocol. Despite the name, it does **not** implement the actual Redis protocol. Uses Rust edition 2024 and tokio.

## Commands

```sh
cargo build                              # build the workspace
cargo check                              # fast type-check
cargo run --example server -p rredis     # run the demo server (binds 127.0.0.1:1234)
cargo run --example client -p rredis     # run the interactive demo client (Get/Set REPL)
cargo test                               # run tests (none exist yet)
```

Building requires `protoc` (the `rredis-wire` build.rs compiles `wire/network_protocol/network_protocol.proto` via prost-build).

## Architecture

Cargo workspace with two crates:

### `wire/` (crate `rredis-wire`) — the wire protocol

All protobuf logic is deliberately encapsulated here; the `rredis` crate never touches prost types. Two layers:

- **`repr`** (`wire/src/repr.rs`): the prost-generated types, included from `OUT_DIR` codegen of `network_protocol.proto`. Private module — never exposed outside the crate.
- **`protocol`** (`wire/src/protocol.rs`): the public, ergonomic Rust types — `Frame<T>`, `Request<T>`, `Data`, and the error hierarchy (`RRError` / `RRErrorKind` in `protocol/error.rs`).

Conversions between the two layers live in dedicated files, one direction each:
- protocol → repr (infallible `From`): `wire/src/protocol/conversions.rs`
- repr → protocol (fallible `TryFrom`, since proto3 makes message fields optional): `wire/src/repr.rs`, written with the `try_to_protocol!` / `cast_or_throw!` / `field_not_optional!` macros from `wire/src/repr/macros.rs`

**Adding a new request type** therefore touches four places: the `.proto` file (note: `Frame.payload` occupies field tag 5, so new `oneof request` variants start at 6+), the `protocol::Request` enum, and both conversion files. Optionally also a `Frame::new_*_request` constructor and the server dispatch in `rredis`.

Everything on the wire is a length-delimited protobuf `Frame`: one `oneof request` plus an optional `Data` payload for extra context. `Data` is the universal value type (bytes, string, ints, array, null).

### `rredis/` — client and server runtime

- **`Connection`** (`rredis/src/connection.rs`): both client- and server-side wrapper over a `TcpStream`; reads/writes length-delimited frames (`read_frame`, `write_frame`, `sendrecv`).
- **Server** (`rredis/src/connection/server.rs` + `server/helper.rs`): `ServerBuilder::new(addr, storage, handler).build().await` → `Server::run()`. The accept loop spawns one tokio task per connection, which decodes frames and dispatches to the handler.
- **Two user-pluggable traits**:
  - `Handler` (`server/handler.rs`): per-request business logic; receives the key/value/payload plus a `StorageSink` to talk to storage.
  - `StorageProxy` (`server/storage.rs`): storage runs as a single actor task. Handlers send `StorageRequest::Get/Set` over an mpsc channel, each carrying a oneshot sender for the reply. `DefaultStorageProxy` is a plain `HashMap` implementation.

The examples in `rredis/examples/` (`server.rs`, `client.rs`) show the intended end-to-end usage of both crates.

## Error handling convention

All fallible code returns `RRError` (kind + optional message), the single error type for the whole project. IO/prost errors are mapped into the appropriate `RRErrorKind` variant (`SerializationError` / `StorageError` / `NetworkError`) rather than propagated raw. Errors are also serializable as frames (`Request::Error`) so the server can send refusals to clients.
