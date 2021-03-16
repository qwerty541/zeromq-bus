use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use rust_playground::RequestData;
use rust_playground::COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use rust_playground::MESSAGE_CONTENT_LENGTH;
use rust_playground::SERVER_ROUTER_SOCKET_ADDR;
use rust_playground::ZEROMQ_FFI_ZERO_FLAG;
use rust_playground::ZEROMQ_MESSAGE_SEND_TIMEOUT_MILLIS;
use std::convert::From;
use std::iter;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use zeromq_ffi::Context;
use zeromq_ffi::Message;
use zeromq_ffi::SocketType;
use zmq as zeromq_ffi;

fn main() {
    env_logger::builder()
        .is_test(true)
        .parse_filters("debug")
        .try_init()
        .expect("failed to initialize environment logger");

    let context = Context::new();
    let mut rng = thread_rng();

    log::debug!("init supported variables");

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
        let message_string = serde_json::json!(RequestData {
            content: iter::repeat(())
                .map(|()| rng.sample(Alphanumeric))
                .map(char::from)
                .take(MESSAGE_CONTENT_LENGTH)
                .collect()
        })
        .to_string();
        let start_send = Instant::now();

        for _ in 1..=COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT {
            sender
                .send(Message::from(message_string.as_str()), ZEROMQ_FFI_ZERO_FLAG)
                .expect("sender failed to send message");
        }

        total_sended += COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;

        log::debug!(
            "{:?} | total sended {} messages",
            SystemTime::now(),
            total_sended
        );

        let send_duration = Instant::now().duration_since(start_send);
        if let Some(left_duration) =
            Duration::from_millis(ZEROMQ_MESSAGE_SEND_TIMEOUT_MILLIS).checked_sub(send_duration)
        {
            thread::sleep(left_duration);
        }
    }
}
