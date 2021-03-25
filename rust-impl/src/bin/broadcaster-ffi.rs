use core::panic;
use lazy_static::lazy_static;
use rust_impl::MessageKind;
use rust_impl::BROADCASTER_PUBLISHERS_SOCKET_ADDRS;
use rust_impl::BROADCASTER_ROUTER_SOCKET_ADDR;
use rust_impl::COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use rust_impl::ZEROMQ_FFI_ZERO_FLAG;
use std::collections::VecDeque;
use std::convert::From;
use std::iter::Iterator;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use zeromq_ffi::Context;
use zeromq_ffi::Message;
use zeromq_ffi::Socket;
use zeromq_ffi::SocketType;
use zeromq_ffi::SNDMORE;
use zmq as zeromq_ffi;

lazy_static! {
    static ref INIT_TIME: Instant = Instant::now();
}

struct BroadcasterPublisherData {
    socket: Socket,
    last_action_time: Instant,
}

fn main() {
    env_logger::builder()
        .is_test(true)
        .parse_filters("debug")
        .try_init()
        .expect("failed to initialize environment logger");

    let mut sended_messages_count: usize = 0;
    let mut sended_errored_messages_count: usize = 0;
    let _ = *INIT_TIME;
    let context = Context::new();
    let mut errored_messages_buffer: VecDeque<Message> = VecDeque::new();

    log::debug!("init supported variables");

    let router_socket = context
        .socket(SocketType::ROUTER)
        .expect("failed to init router socket");

    log::debug!("init broadcaster router socket");

    router_socket
        .bind(BROADCASTER_ROUTER_SOCKET_ADDR.as_str())
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

    let mut publishers: Vec<BroadcasterPublisherData> =
        Vec::with_capacity(BROADCASTER_PUBLISHERS_SOCKET_ADDRS.len());

    for publisher_addr in BROADCASTER_PUBLISHERS_SOCKET_ADDRS.iter() {
        let x_pub_socket = context
            .socket(SocketType::XPUB)
            .expect("failed to init x pub socket");

        x_pub_socket
            .bind(publisher_addr.as_str())
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

    log::debug!("running messages processing loop");

    loop {
        let mut message_kind = MessageKind::default();
        let message = if let Some(errored_message) = errored_messages_buffer.pop_front() {
            message_kind = MessageKind::Errored;
            errored_message
        } else {
            let mut message = Message::new();

            router_socket
                .recv(&mut message, ZEROMQ_FFI_ZERO_FLAG)
                .expect("failed to recv message");

            message
        };

        let should_send_more_parts = message.get_more();
        let mut index_of_publisher_that_will_be_used = 0;
        let mut max_duration_since_last_action = Duration::from_nanos(0_u64);

        for (
            index,
            BroadcasterPublisherData {
                last_action_time, ..
            },
        ) in publishers.iter().enumerate()
        {
            let current_duration_since_last_action = (*last_action_time).duration_since(*INIT_TIME);
            if current_duration_since_last_action > max_duration_since_last_action {
                max_duration_since_last_action = current_duration_since_last_action;
                index_of_publisher_that_will_be_used = index;
            }
        }

        let bytes_slice_cloned_from_message = &*message;
        let cloned_message = Message::from(bytes_slice_cloned_from_message);

        match publishers[index_of_publisher_that_will_be_used]
            .socket
            .send(
                cloned_message,
                if should_send_more_parts {
                    SNDMORE
                } else {
                    ZEROMQ_FFI_ZERO_FLAG
                },
            ) {
            Ok(()) => {
                if !should_send_more_parts {
                    match message_kind {
                        MessageKind::Incoming => {
                            sended_messages_count += 1;
                        }
                        MessageKind::Errored => {
                            sended_errored_messages_count += 1;
                        }
                    }

                    let total_processed = sended_messages_count + sended_errored_messages_count;

                    if total_processed
                        % COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT
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
            }
            Err(e) => {
                log::error!("failed to send message because of: {}", e);

                errored_messages_buffer.push_back(message);
            }
        }

        publishers[index_of_publisher_that_will_be_used].last_action_time = Instant::now();
    }
}
