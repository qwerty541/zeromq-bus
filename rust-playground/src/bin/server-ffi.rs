use core::panic;
use lazy_static::lazy_static;
use rust_playground::COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT;
use rust_playground::SERVER_PUBLISHER_SOCKET_ADDRS;
use rust_playground::SERVER_ROUTER_SOCKET_ADDR;
use rust_playground::ZEROMQ_FFI_ZERO_FLAG;
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

struct ServerPublisherData {
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
    let _ = *INIT_TIME;

    log::debug!("init supported variables");

    let context = Context::new();

    let router_socket = context
        .socket(SocketType::ROUTER)
        .expect("failed to init router socket");

    router_socket
        .bind(SERVER_ROUTER_SOCKET_ADDR.as_str())
        .expect("failed to bind server router socket");

    log::debug!("init server router socket");

    let mut publishers: Vec<ServerPublisherData> =
        Vec::with_capacity(SERVER_PUBLISHER_SOCKET_ADDRS.len());

    for publisher_addr in SERVER_PUBLISHER_SOCKET_ADDRS.iter() {
        let x_pub_socket = context
            .socket(SocketType::XPUB)
            .expect("failed to init x pub socket");

        x_pub_socket
            .bind(publisher_addr.as_str())
            .unwrap_or_else(|error| {
                panic!(
                    "binding server publisher socket on '{}' failed with: {}",
                    publisher_addr, error
                )
            });

        publishers.push(ServerPublisherData {
            socket: x_pub_socket,
            last_action_time: Instant::now(),
        });
    }

    log::debug!("init server publisher sockets");

    log::debug!("running messages processing loop");

    loop {
        let mut message = Message::new();

        router_socket
            .recv(&mut message, ZEROMQ_FFI_ZERO_FLAG)
            .expect("failed to recv message");

        let should_send_more_parts = message.get_more();
        let mut index_of_publisher_that_will_be_used = 0;
        let mut max_duration_since_last_action = Duration::from_nanos(0_u64);

        for (
            index,
            ServerPublisherData {
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

        publishers[index_of_publisher_that_will_be_used]
            .socket
            .send(
                message,
                if should_send_more_parts {
                    SNDMORE
                } else {
                    ZEROMQ_FFI_ZERO_FLAG
                },
            )
            .expect("failed to send message");

        if should_send_more_parts {
            sended_messages_count += 1;

            if sended_messages_count
                % COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT
                == 0
            {
                log::debug!(
                    "{:?} | server processed {} messages",
                    SystemTime::now(),
                    sended_messages_count
                );
            }
        }

        publishers[index_of_publisher_that_will_be_used].last_action_time = Instant::now();
    }
}
