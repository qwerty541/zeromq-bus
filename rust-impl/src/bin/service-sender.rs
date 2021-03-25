use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use rust_impl::RequestData;
use rust_impl::COUNT_OF_ZEROMQ_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use rust_impl::MESSAGE_CONTENT_LENGTH;
use rust_impl::BROADCASTER_ROUTER_SOCKET_ADDR;
use rust_impl::ZEROMQ_MESSAGE_SEND_TIMEOUT_MILLIS;
use std::convert::From;
use std::iter;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use tokio::time::sleep;
use zeromq::DealerSocket;
use zeromq::Socket;
use zeromq::SocketSend;
use zeromq::ZmqMessage;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::builder()
        .is_test(true)
        .parse_filters("debug")
        .try_init()
        .expect("failed to initialize environment logger");

    let mut sender = DealerSocket::new();
    let mut rng = thread_rng();

    log::debug!("init sender");

    sender
        .connect(BROADCASTER_ROUTER_SOCKET_ADDR.as_str())
        .await
        .expect("failed to connect to broadcaster router socket.");

    log::debug!("sender has connected to broadcaster");

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

        for _ in 1..=COUNT_OF_ZEROMQ_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT {
            sender
                .send(ZmqMessage::from(message_string.as_str()))
                .await
                .expect("sender failed to send message");
        }

        total_sended += COUNT_OF_ZEROMQ_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;

        log::debug!(
            "{:?} | total sended {} messages",
            SystemTime::now(),
            total_sended
        );

        let send_duration = Instant::now().duration_since(start_send);
        if let Some(left_duration) =
            Duration::from_millis(ZEROMQ_MESSAGE_SEND_TIMEOUT_MILLIS).checked_sub(send_duration)
        {
            sleep(left_duration).await;
        }
    }
}
