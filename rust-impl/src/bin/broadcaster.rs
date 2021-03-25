// select! macro used in messages processign loop relies on proc-macro-hack,
// and require to set the compiler's recursion limit very high.
#![recursion_limit = "1024"]

use core::panic;
use futures::select;
use futures::FutureExt;
use lazy_static::lazy_static;
use rust_impl::MessageKind;
use rust_impl::BROADCASTER_PUBLISHERS_SOCKET_ADDRS;
use rust_impl::BROADCASTER_ROUTER_SOCKET_ADDR;
use rust_impl::COUNT_OF_ZEROMQ_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use std::iter::Iterator;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use zeromq::PubSocket;
use zeromq::RouterSocket;
use zeromq::Socket;
use zeromq::SocketRecv;
use zeromq::SocketSend;
use zeromq::ZmqMessage;
use zeromq::ZmqResult;

lazy_static! {
    static ref INIT_TIME: Instant = Instant::now();
}

struct BroadcasterPublisherData {
    socket: PubSocket,
    last_action_time: Instant,
}

fn main() {
    // Init environment logger.
    env_logger::builder()
        .is_test(true)
        .parse_filters("debug")
        .try_init()
        .expect("failed to initialize environment logger");

    // Init tokio runtime.
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    // Run application.
    runtime.block_on(async move {
        // Init supported variables.
        let mut sended_messages_count: usize = 0;
        let mut sended_errored_messages_count: usize = 0;
        let _ = *INIT_TIME;

        log::debug!("init supported variables");

        // Init broadcaster router socket.
        let mut router_socket = RouterSocket::new();

        log::debug!("init broadcaster router socket");

        router_socket
            .bind(BROADCASTER_ROUTER_SOCKET_ADDR.as_str())
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "binding broadcaster router socket on {} failed with: {}",
                    BROADCASTER_ROUTER_SOCKET_ADDR.as_str(),
                    error
                )
            });

        log::debug!(
            "broadcaster router socket binded on {}",
            BROADCASTER_ROUTER_SOCKET_ADDR.as_str()
        );

        // Init broadcaster publisher sockets.
        let mut publishers: Vec<BroadcasterPublisherData> =
            Vec::with_capacity(BROADCASTER_PUBLISHERS_SOCKET_ADDRS.len());

        for publisher_addr in BROADCASTER_PUBLISHERS_SOCKET_ADDRS.iter() {
            let mut x_pub_socket = PubSocket::new();

            x_pub_socket
                .bind(publisher_addr.as_str())
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "binding broadcaster publisher socket on '{}' failed with: {}",
                        publisher_addr, error
                    )
                });

            publishers.push(BroadcasterPublisherData {
                socket: x_pub_socket,
                last_action_time: Instant::now(),
            });
        }

        log::debug!(
            "init broadcaster publisher sockets and binded on {}",
            BROADCASTER_PUBLISHERS_SOCKET_ADDRS.join(", ")
        );

        // Init channel for errored messages.
        let (errored_messages_channel_sender, mut errored_messages_channel_receiver) =
            mpsc::unbounded_channel::<ZmqMessage>();

        log::debug!("init channel for errored messages");

        log::debug!("running messages processing loop");

        // Messages processing loop.
        'messages_processing: loop {
            let mut message_kind = MessageKind::default();

            // Receive message from router socket or channel for errored messages.
            let message = select! {
                maybe_message = router_socket.recv().fuse() => {
                    match maybe_message {
                        Ok(message) => message,
                        Err(e) => {
                            log::error!("router socket failed to receive message: {}", e);
                            continue 'messages_processing;
                        },
                    }
                },
                maybe_errored_message = errored_messages_channel_receiver.recv().fuse() => {
                    match maybe_errored_message {
                        Some(message) => {
                            message_kind = MessageKind::Errored;
                            message
                        },
                        None => {
                            panic!("errored messages sender dropped");
                        }
                    }
                }
            };

            // Define index of publisher that will be used for sending.
            let mut index_of_publisher_that_will_be_used = 0;
            let mut max_duration_since_last_action = Duration::from_nanos(0_u64);
            for (
                index,
                BroadcasterPublisherData {
                    last_action_time, ..
                },
            ) in publishers.iter().enumerate()
            {
                let current_duration_since_last_action =
                    (*last_action_time).duration_since(*INIT_TIME);
                if current_duration_since_last_action > max_duration_since_last_action {
                    max_duration_since_last_action = current_duration_since_last_action;
                    index_of_publisher_that_will_be_used = index;
                }
            }

            // Send message to subscribers.
            match publishers[index_of_publisher_that_will_be_used]
                .socket
                .send(message.clone())
                .await
            {
                ZmqResult::Ok(()) => {
                    match message_kind {
                        MessageKind::Incoming => {
                            sended_messages_count += 1;
                        }
                        MessageKind::Errored => {
                            sended_errored_messages_count += 1;
                        }
                    }
                    let total_processed = sended_messages_count + sended_errored_messages_count;

                    if total_processed % COUNT_OF_ZEROMQ_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT
                        == 0
                    {
                        log::debug!(
                            "{:?} | processed {} messages\n ({} incoming, {} errored)",
                            SystemTime::now(),
                            total_processed,
                            sended_messages_count,
                            sended_errored_messages_count
                        );
                    }
                }
                ZmqResult::Err(e) => {
                    log::error!("failed to send message because of: {}", e);

                    errored_messages_channel_sender
                        .send(message)
                        .expect("errored message channel receiver droppped");
                }
            }

            publishers[index_of_publisher_that_will_be_used].last_action_time = Instant::now();
        }
    });
}
