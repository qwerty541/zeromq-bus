use core::panic;
use rust_impl::COUNT_OF_ZEROMQ_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use rust_impl::SERVER_PUBLISHER_SOCKET_ADDRS;
use std::time::SystemTime;
use zeromq::Socket;
use zeromq::SocketRecv;
use zeromq::SubSocket;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Init environment logger.
    env_logger::builder()
        .is_test(true)
        .parse_filters("debug")
        .try_init()
        .expect("failed to initialize environment logger");

    let mut rep_socket = SubSocket::new();

    log::debug!("init receiver");

    for publisher_addr in SERVER_PUBLISHER_SOCKET_ADDRS.iter() {
        rep_socket
            .connect(publisher_addr.as_str())
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "connection to server dealer socket '{}' failed with: {}",
                    publisher_addr, error
                )
            });

        rep_socket.subscribe("").await.unwrap_or_else(|error| {
            panic!(
                "subscription to server dealer socket '{}' failed with: {}",
                publisher_addr, error
            )
        });
    }

    log::debug!("receiver connected to all publishers");

    let mut total_received = 0;
    'receive_messages: loop {
        let _message = match rep_socket.recv().await {
            Ok(message) => message,
            Err(e) => {
                log::error!("failed to receive message: {}", e);
                continue 'receive_messages;
            }
        };

        total_received += 1;

        if total_received % COUNT_OF_ZEROMQ_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT == 0 {
            log::debug!(
                "{:?} | received {} messages",
                SystemTime::now(),
                total_received
            );
        }
    }
}
