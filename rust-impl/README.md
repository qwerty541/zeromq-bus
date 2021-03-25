## Implementation of broadcast pattern using rust implementation of ZeroMQ.

### Commands to launch broadcaster/services builded with [ZeroMQ lib natively implemented on Rust](https://github.com/zeromq/zmq.rs).

```
cargo run --bin broadcaster

cargo run --bin service-receiver

cargo run --bin service-sender
```

### Commands to launch broadcaster/services builded with [ZeroMQ Rust FFI lib](https://github.com/erickt/rust-zmq).

```
cargo run --bin broadcaster-ffi

cargo run --bin service-receiver-ffi

cargo run --bin service-sender-ffi
```
