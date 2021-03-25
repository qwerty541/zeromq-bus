use core::panic;
use rust_impl::COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use rust_impl::SERVER_PUBLISHER_SOCKET_ADDRS;
use rust_impl::ZEROMQ_FFI_ZERO_FLAG;
use std::time::SystemTime;
use zeromq_ffi::Context;
use zeromq_ffi::Message;
use zeromq_ffi::SocketType;
use zmq as zeromq_ffi;

fn main() {
    // Init environment logger.
    env_logger::builder()
        .is_test(true)
        .parse_filters("debug")
        .try_init()
        .expect("failed to initialize environment logger");

    let context = Context::new();

    let socket = context
        .socket(SocketType::SUB)
        .expect("failed to init subscriber socket");

    log::debug!("init receiver");

    for publisher_addr in SERVER_PUBLISHER_SOCKET_ADDRS.iter() {
        socket
            .connect(publisher_addr.as_str())
            .unwrap_or_else(|error| {
                panic!(
                    "connection to server dealer socket '{}' failed with: {}",
                    publisher_addr, error
                )
            });

        socket.set_subscribe(b"").unwrap_or_else(|error| {
            panic!(
                "subscription to server dealer socket '{}' failed with: {}",
                publisher_addr, error
            )
        });
    }

    log::debug!("receiver connected to all publishers");

    let mut total_received = 0;
    loop {
        let mut message = Message::new();

        socket
            .recv(&mut message, ZEROMQ_FFI_ZERO_FLAG)
            .expect("failed to receive message");

        if !message.get_more() {
            total_received += 1;

            if total_received % COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT == 0
            {
                log::debug!(
                    "{:?} | received {} messages",
                    SystemTime::now(),
                    total_received
                );
            }
        }
    }
}
