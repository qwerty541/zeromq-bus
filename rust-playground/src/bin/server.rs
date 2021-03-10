#![recursion_limit = "1024"]

use futures::select;
use futures::FutureExt;
use lazy_static::lazy_static;
use rust_playground::MessageKind;
use rust_playground::ServerPublisherData;
use rust_playground::COUNT_OF_COMMANDS_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use rust_playground::SERVER_PUBLISHER_SOCKET_ADDRS;
use rust_playground::SERVER_ROUTER_SOCKET_ADDR;
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

        // Init server router socket.
        let mut router_socket = RouterSocket::new();

        router_socket
            .bind(SERVER_ROUTER_SOCKET_ADDR.as_str())
            .await
            .expect("failed to bind server router socket");

        log::debug!("init server router socket");

        // Init server publisher sockets.
        let mut publishers: Vec<ServerPublisherData> =
            Vec::with_capacity(SERVER_PUBLISHER_SOCKET_ADDRS.len());

        for publisher_addr in SERVER_PUBLISHER_SOCKET_ADDRS.iter() {
            let mut x_pub_socket = PubSocket::new();

            x_pub_socket
                .bind(publisher_addr.as_str())
                .await
                .expect("failed to bind server publisher socket");

            publishers.push(ServerPublisherData {
                socket: x_pub_socket,
                last_action_time: Instant::now(),
            });
        }

        log::debug!("init server publisher sockets");

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
                ServerPublisherData {
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
                ZmqResult::Ok(()) => match message_kind {
                    MessageKind::Incoming => {
                        sended_messages_count += 1;

                        if sended_messages_count
                            % COUNT_OF_COMMANDS_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT
                            == 0
                        {
                            log::debug!(
                                "{:?} | server send {} messages",
                                SystemTime::now(),
                                sended_messages_count
                            );
                        }
                    }
                    MessageKind::Errored => {
                        sended_errored_messages_count += 1;

                        if sended_errored_messages_count
                            % COUNT_OF_COMMANDS_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT
                            == 0
                        {
                            log::debug!(
                                "{:?} | server send {} errored messages",
                                SystemTime::now(),
                                sended_errored_messages_count
                            );
                        }
                    }
                },
                ZmqResult::Err(e) => {
                    log::error!("server failed to send message: {}", e);

                    errored_messages_channel_sender
                        .send(message)
                        .expect("errored message channel receiver droppped");
                }
            }

            publishers[index_of_publisher_that_will_be_used].last_action_time = Instant::now();
        }
    });
}
