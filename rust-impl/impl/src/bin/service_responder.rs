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
use rust_impl::BUS_PUBLISHERS_SOCKET_ADDRS;
use rust_impl::BUS_ROUTER_SOCKET_ADDR;
use rust_impl::LOG_LEVEL;
use rust_impl::REQUESTS_COUNT_INSIDE_ONE_GROUP;
use rust_impl::RUST_LOG_ENVIRONMENT_VARIABLE_NAME;
use rust_impl::ZEROMQ_ZERO_FLAG;
use std::convert::From;
use std::env;
use std::time::SystemTime;
use zeromq_messages::codec::decode_message_kind;
use zeromq_messages::codec::decode_message_payload;
use zeromq_messages::codec::decode_message_uuid;
use zeromq_messages::codec::encode_message;
use zeromq_messages::kind::ZeromqMessageKind;
use zeromq_messages::messages::ValueMultiplicationRequest;
use zeromq_messages::messages::ValueMultiplicationResponse;
use zmq::Context;
use zmq::Message;
use zmq::SocketType;

#[allow(clippy::too_many_lines)]
fn main() {
    if env::var(RUST_LOG_ENVIRONMENT_VARIABLE_NAME).is_err() {
        env::set_var(RUST_LOG_ENVIRONMENT_VARIABLE_NAME, LOG_LEVEL);
    }

    env_logger::init();

    let context = Context::new();

    let sender = context
        .socket(SocketType::DEALER)
        .expect("failed to initialize sender socket");

    log::debug!("initialized sender socket");

    sender
        .connect(BUS_ROUTER_SOCKET_ADDR.as_str())
        .expect("failed to connect to BUS router socket.");

    log::debug!("sender has connected to BUS router socket");

    let receiver = context
        .socket(SocketType::SUB)
        .expect("failed to initialize receiver socket");

    log::debug!("initialized receiver socket");

    for publisher_address in BUS_PUBLISHERS_SOCKET_ADDRS.iter() {
        receiver
            .connect(publisher_address.as_str())
            .unwrap_or_else(|error| {
                panic!(
                    "connection to BUS publisher socket '{}' failed with: {}",
                    publisher_address, error
                )
            });

        receiver.set_subscribe(b"").unwrap_or_else(|error| {
            panic!(
                "subscription to BUS publisher socket '{}' failed with: {}",
                publisher_address, error
            )
        });
    }

    log::debug!(
        "receiver has connected to all BUS publishers: {}",
        BUS_PUBLISHERS_SOCKET_ADDRS.join(", ")
    );

    let mut total_processed_messages_count = 0;

    'messages_processing: loop {
        let message_bytes = match receiver.recv_bytes(ZEROMQ_ZERO_FLAG) {
            Ok(message_bytes) => message_bytes,
            Err(error) => {
                log::error!("failed to receive message because of: {}", error);
                continue 'messages_processing;
            }
        };

        log::trace!("< {:?}", message_bytes);

        let (message_kind, message_bytes_without_kind) =
            match decode_message_kind(message_bytes) {
                Ok(message_kind_and_left_bytes) => message_kind_and_left_bytes,
                Err(error) => {
                    log::error!("failed to decode message kind because of: {}", error);
                    continue 'messages_processing;
                }
            };

        if !(matches!(message_kind, ZeromqMessageKind::ValueMultiplicationRequest)) {
            log::trace!("ignored message with unexpected kind {:?}", message_kind);
            continue 'messages_processing;
        }

        let (uuid, message_payload_bytes) = decode_message_uuid(message_bytes_without_kind);
        let payload = match decode_message_payload::<'_, ValueMultiplicationRequest>(
            message_payload_bytes.as_slice(),
        ) {
            Ok(payload) => payload,
            Err(error) => {
                log::error!("failed to decode message payload because of: {}", error);
                continue 'messages_processing;
            }
        };

        let response_message_bytes = match encode_message(
            uuid,
            ValueMultiplicationResponse {
                result: payload.value * payload.multiplier,
            },
        ) {
            Ok(message) => message,
            Err(error) => {
                log::error!("failed to encode message because of: {}", error);
                continue 'messages_processing;
            }
        };

        if let Err(error) = sender.send(
            Message::from(response_message_bytes.clone()),
            ZEROMQ_ZERO_FLAG,
        ) {
            log::error!("failed to send message because of: {}", error);
            continue 'messages_processing;
        }

        log::trace!("> {:?}", response_message_bytes);

        total_processed_messages_count += 1;

        if total_processed_messages_count % REQUESTS_COUNT_INSIDE_ONE_GROUP == 0 {
            log::debug!(
                "{:?} | total processed {} messages",
                SystemTime::now(),
                total_processed_messages_count
            );
        }
    }
}
