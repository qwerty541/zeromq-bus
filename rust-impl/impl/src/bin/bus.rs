// Rust flags
#![warn(nonstandard_style)]
#![warn(future_incompatible)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]
#![recursion_limit = "1024"]
// Clippy flags
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

use core::panic;
use lazy_static::lazy_static;
use rust_impl::BusPublisherData;
use rust_impl::BUS_PUBLISHERS_SOCKET_ADDRS;
use rust_impl::BUS_ROUTER_SOCKET_ADDR;
use rust_impl::LOG_LEVEL;
use rust_impl::REQUESTS_COUNT_INSIDE_ONE_GROUP;
use rust_impl::RUST_LOG_ENVIRONMENT_VARIABLE_NAME;
use rust_impl::ZEROMQ_ZERO_FLAG;
use std::collections::VecDeque;
use std::convert::From;
use std::env;
use std::iter::Iterator;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use zmq::Context;
use zmq::Message;
use zmq::SocketType;

lazy_static! {
    static ref INIT_TIME: Instant = Instant::now();
}

#[allow(clippy::too_many_lines)]
fn main() {
    if env::var(RUST_LOG_ENVIRONMENT_VARIABLE_NAME).is_err() {
        env::set_var(RUST_LOG_ENVIRONMENT_VARIABLE_NAME, LOG_LEVEL);
    }

    env_logger::init();

    // initialize lazy static variable
    let _ = *INIT_TIME;

    let context = Context::new();
    let mut errored_messages_bytes_buffer: VecDeque<Vec<u8>> = VecDeque::new();

    let router_socket = context
        .socket(SocketType::ROUTER)
        .expect("failed to initialize BUS router socket");

    log::debug!("initialized BUS router socket");

    router_socket
        .bind(BUS_ROUTER_SOCKET_ADDR.as_str())
        .unwrap_or_else(|error| {
            panic!(
                "binding BUS router socket on {} failed with: {}",
                BUS_ROUTER_SOCKET_ADDR.as_str(),
                error
            )
        });

    log::debug!(
        "BUS router socket binded on {}",
        BUS_ROUTER_SOCKET_ADDR.as_str()
    );

    let mut publishers: Vec<BusPublisherData> =
        Vec::with_capacity(BUS_PUBLISHERS_SOCKET_ADDRS.len());

    for publisher_address in BUS_PUBLISHERS_SOCKET_ADDRS.iter() {
        let publisher = context
            .socket(SocketType::XPUB)
            .expect("failed to initialize one of BUS publisher sockets");

        publisher
            .bind(publisher_address.as_str())
            .unwrap_or_else(|error| {
                panic!(
                    "binding BUS publisher socket on {} failed with: {}",
                    publisher_address, error
                )
            });

        publishers.push(BusPublisherData::new(publisher));
    }

    log::debug!(
        "initialized BUS publisher sockets and binded on {}",
        BUS_PUBLISHERS_SOCKET_ADDRS.join(", ")
    );

    let mut total_processed_messages_count: usize = 0;
    let (received_messages_channel_sender, received_messages_channel_receiver) =
        mpsc::channel::<Vec<u8>>();

    log::debug!("running sender thread");
    drop(thread::spawn(move || {
        #[allow(unused_labels)]
        'messages_sender: loop {
            let message_bytes = errored_messages_bytes_buffer.pop_front().map_or_else(
                || {
                    received_messages_channel_receiver
                        .recv()
                        .expect("received messages mpsc sender dropped")
                },
                |errored_message_bytes| errored_message_bytes,
            );

            let mut index_of_publisher_that_will_be_used = 0;
            let mut max_duration_since_last_action = Duration::from_nanos(0_u64);

            for (index, publisher) in publishers.iter().enumerate() {
                let current_duration_since_last_action =
                    publisher.get_last_action_time().duration_since(*INIT_TIME);
                if current_duration_since_last_action > max_duration_since_last_action {
                    max_duration_since_last_action = current_duration_since_last_action;
                    index_of_publisher_that_will_be_used = index;
                }
            }

            match (*publishers[index_of_publisher_that_will_be_used])
                .send(Message::from(message_bytes.clone()), ZEROMQ_ZERO_FLAG)
            {
                Ok(()) => {
                    log::trace!("> {:?}", message_bytes);
                    total_processed_messages_count += 1;
                }
                Err(error) => {
                    log::error!("failed to send message because of: {}", error);
                    errored_messages_bytes_buffer.push_back(message_bytes);
                }
            }

            publishers[index_of_publisher_that_will_be_used].update_last_action_time();

            if total_processed_messages_count % REQUESTS_COUNT_INSIDE_ONE_GROUP == 0 {
                log::debug!(
                    "{:?} | total processed {} messages",
                    SystemTime::now(),
                    total_processed_messages_count
                );
            }
        }
    }));

    log::debug!("running received loop");
    'messages_receiver: loop {
        // Firstly receive first message frame which is the sender identity.
        let identity_bytes = match router_socket.recv_bytes(ZEROMQ_ZERO_FLAG) {
            Ok(identity_bytes) => identity_bytes,
            Err(error) => {
                log::error!("failed to receive sender identity because of: {}", error);
                continue 'messages_receiver;
            }
        };

        log::trace!("< [IDENTITY] {:?}", identity_bytes);

        // In case if we succeed to receive identity try to receive next frame which
        // include message content.
        let message_bytes = match router_socket.recv_bytes(ZEROMQ_ZERO_FLAG) {
            Ok(message_bytes) => message_bytes,
            Err(error) => {
                log::error!("failed to receive message because of: {}", &error);
                continue 'messages_receiver;
            }
        };

        log::trace!("< {:?}", message_bytes);

        received_messages_channel_sender
            .send(message_bytes)
            .expect("received messages mpsc receiver dropped");
    }
}
