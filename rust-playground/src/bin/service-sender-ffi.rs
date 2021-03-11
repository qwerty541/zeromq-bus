use rust_playground::COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use rust_playground::SERVER_ROUTER_SOCKET_ADDR;
use rust_playground::ZEROMQ_FFI_FLAG;
use rust_playground::ZEROMQ_MESSAGE_SEND_TIMEOUT_MILLIS;
use std::convert::From;
use std::thread;
use std::time::Duration;
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

    let sender = context
        .socket(SocketType::DEALER)
        .expect("failed to init dealer socket");

    log::debug!("init sender");

    sender
        .connect(SERVER_ROUTER_SOCKET_ADDR.as_str())
        .expect("failed to connect to server router socket.");

    log::debug!("sender connected");

    let mut total_sended = 0;
    loop {
        for _ in 1..=COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT {
            sender
                .send(Message::from(Vec::new()), ZEROMQ_FFI_FLAG)
                .expect("client-sender failed to send message");
        }

        total_sended += COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;

        log::debug!(
            "{:?} | total sended {} messages",
            SystemTime::now(),
            total_sended
        );

        thread::sleep(Duration::from_millis(ZEROMQ_MESSAGE_SEND_TIMEOUT_MILLIS));
    }
}
