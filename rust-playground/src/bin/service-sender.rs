// use rand::distributions::Alphanumeric;
// use rand::thread_rng;
// use rand::Rng;
// use rust_playground::RequestData;
use rust_playground::COMMANDS_SEND_TIMEOUT_MILLIS;
use rust_playground::COUNT_OF_COMMANDS_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
// use rust_playground::MESSAGE_CONTENT_LENGTH;
use rust_playground::SERVER_ROUTER_SOCKET_ADDR;
// use std::iter;
use std::time::Duration;
use std::time::SystemTime;
use tokio::time::sleep;
use zeromq::DealerSocket;
use zeromq::Socket;
use zeromq::SocketSend;
use zeromq::ZmqMessage;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Init environment logger.
    env_logger::builder()
        .is_test(true)
        .parse_filters("debug")
        .try_init()
        .expect("failed to initialize environment logger");

    let mut sender = DealerSocket::new();
    // let mut rng = thread_rng();

    log::debug!("init sender");

    sender
        .connect(SERVER_ROUTER_SOCKET_ADDR.as_str())
        .await
        .expect("failed to connect to server router socket.");

    log::debug!("sender connected");

    let mut total_sended = 0;
    loop {
        for _ in 1..=COUNT_OF_COMMANDS_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT {
            // let message_string = serde_json::json!(RequestData {
            //     content: iter::repeat(())
            //         .map(|()| rng.sample(Alphanumeric))
            //         .map(char::from)
            //         .take(MESSAGE_CONTENT_LENGTH)
            //         .collect()
            // })
            // .to_string();

            sender
                .send(ZmqMessage::from(String::new()))
                .await
                .expect("client-sender failed to send message");

            // log::debug!("send message: {:?}", message_string);
        }

        total_sended += COUNT_OF_COMMANDS_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;

        log::debug!(
            "{:?} | total sended {} messages",
            SystemTime::now(),
            total_sended
        );

        sleep(Duration::from_millis(COMMANDS_SEND_TIMEOUT_MILLIS)).await;
    }
}
