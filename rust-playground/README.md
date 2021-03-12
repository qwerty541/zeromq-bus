# Playground for experimentation with rust implementation of ZeroMQ.

### Commands to launch server/services builded with (ZeroMQ lib natively implemented on Rust)[https://github.com/zeromq/zmq.rs].

```
cargo run --bin server

cargo run --bin service-receiver

cargo run --bin service-sender
```

### Commands to launch server/services builded with (ZeroMQ Rust FFI lib)[https://github.com/erickt/rust-zmq]

```
cargo run --bin server-ffi

cargo run --bin service-receiver-ffi

cargo run --bin service-sender-ffi
```
