use bytes::Buf;
use core::panic;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;
use std::convert::From;
use std::iter;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::watch;
use tokio::time::sleep;
use zeromq::DealerSocket;
use zeromq::PubSocket;
use zeromq::RouterSocket;
use zeromq::Socket;
use zeromq::SocketRecv;
use zeromq::SocketSend;
use zeromq::SubSocket;
use zeromq::ZmqMessage;

macro_rules! format_endpoint {
    ($endpoint:expr) => {
        format!("tcp://{}", $endpoint)
    };
}

lazy_static::lazy_static! {
    static ref SERVER_ROUTER_SOCKET_ADDR: String = format_endpoint!("0.0.0.0:56737");
    static ref SERVER_PUBLISHER_SOCKET_ADDR: String = format_endpoint!("0.0.0.0:56738");
}

const SERVICES_COUNT: usize = 5;
const COMMANDS_SHOULD_BE_SEND_BY_SERVICE_REQ_SOCKET: usize = 1;
const COMMANDS_SEND_TIMEOUT_MILLIS: u64 = 1000;
const MPSC_CHANNEL_BUFFER_SIZE: usize = 10;
const MESSAGE_CONTENT_LENGTH: usize = 32;

const ROUTER_READINESS_KEY_DEFAULT: usize = 0;
const ROUTER_READINESS_KEY_READY: usize = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RequestData {
    content: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Some supporting variables.
    let (start_messaging_notifier_sender, start_messaging_notifier_receiver) =
        watch::channel(ROUTER_READINESS_KEY_DEFAULT);
    let (services_rep_sockets_ready_sender, mut services_rep_sockets_ready_receiver) =
        mpsc::channel::<()>(MPSC_CHANNEL_BUFFER_SIZE);

    // Init server router socket.
    let mut router_socket = RouterSocket::new();

    router_socket
        .bind(SERVER_ROUTER_SOCKET_ADDR.as_str())
        .await
        .expect("failed to bind server router socket");

    println!("init server server router socket");

    // Init server publisher socket.
    let mut x_pub_socket = PubSocket::new();

    x_pub_socket
        .bind(SERVER_PUBLISHER_SOCKET_ADDR.as_str())
        .await
        .expect("failed to bind server publisher socket");

    println!("init server server publisher socket");

    // Init client sender sockets in separate threads.
    for number in 1..=SERVICES_COUNT {
        let builder = thread::Builder::new().name(format!("client-sender-{}", number));
        let mut cloned_start_messaging_notifier_receiver =
            start_messaging_notifier_receiver.clone();

        builder
            .spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .thread_name(format!("client-sender-{}", number))
                    .build()
                    .expect("failde to build tokio runtime.")
                    .block_on(async move {
                        let mut sender = DealerSocket::new();
                        let mut rng = thread_rng();

                        sender
                            .connect(SERVER_ROUTER_SOCKET_ADDR.as_str())
                            .await
                            .expect("failed to connect to server router socket.");

                        println!("init client-sender-{}", number);

                        cloned_start_messaging_notifier_receiver
                            .changed()
                            .await
                            .expect("failed to get signal about router socket set up.");

                        loop {
                            for _ in 1..=COMMANDS_SHOULD_BE_SEND_BY_SERVICE_REQ_SOCKET {
                                let message_string = serde_json::json!(RequestData {
                                    content: iter::repeat(())
                                        .map(|()| rng.sample(Alphanumeric))
                                        .map(char::from)
                                        .take(MESSAGE_CONTENT_LENGTH)
                                        .collect()
                                })
                                .to_string();

                                sender
                                    .send(ZmqMessage::from(message_string.clone()))
                                    .await
                                    .expect("client-sender failed to send message");

                                println!(
                                    "client-sender-{} send message: {:?}",
                                    number, message_string
                                );
                            }

                            sleep(Duration::from_millis(COMMANDS_SEND_TIMEOUT_MILLIS)).await;
                        }
                    });
            })
            .unwrap_or_else(|e| {
                panic!("failed to spawn thread for client-sender-{}: {}", number, e)
            });
    }

    // Init client receiver sockets in separate threads.
    for number in 1..=SERVICES_COUNT {
        let builder = thread::Builder::new().name(format!("client-receiver-{}", number));
        let cloned_services_rep_sockets_ready_sender = services_rep_sockets_ready_sender.clone();

        builder
            .spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .thread_name(format!("client-receiver-{}", number))
                    .build()
                    .expect("failde to build tokio runtime.")
                    .block_on(async move {
                        let mut rep_socket = SubSocket::new();

                        rep_socket
                            .connect(SERVER_PUBLISHER_SOCKET_ADDR.as_str())
                            .await
                            .expect("failed to connect to server dealer socket.");

                        rep_socket
                            .subscribe("")
                            .await
                            .expect("failed to subscriner to server dealer");

                        cloned_services_rep_sockets_ready_sender
                            .send(())
                            .await
                            .expect("failed to send signal about one of rep sockets init.");

                        println!("init client-receiver-{}", number);

                        'client_receive_messages: loop {
                            let mut message = match rep_socket.recv().await {
                                Ok(message) => message.into_vecdeque(),
                                Err(e) => {
                                    eprintln!(
                                        "client-receiver-{} failed to receive message: {}",
                                        number, e
                                    );
                                    continue 'client_receive_messages;
                                }
                            };
                            let message_bytes = message
                                .pop_back()
                                .expect("rep socket received message without required data.");
                            let message_string = unsafe {
                                String::from_utf8_unchecked(message_bytes.bytes().to_vec())
                            };

                            println!(
                                "client-receiver-{} incoming message: {:?}",
                                number, message_string
                            );
                        }
                    });
            })
            .unwrap_or_else(|e| {
                panic!(
                    "failed to spawn thread for client-receiver-{}: {}",
                    number, e
                )
            });
    }

    // Wait for client subscribers init.
    let mut initialized_subscribers_count: usize = 0;
    while initialized_subscribers_count < SERVICES_COUNT {
        services_rep_sockets_ready_receiver.recv().await;
        initialized_subscribers_count += 1;
    }

    // Notify to start messaging.
    start_messaging_notifier_sender
        .send(ROUTER_READINESS_KEY_READY)
        .expect("failed to send signal abount subscriber set up.");

    // Server messages processing loop.
    'receive_messages: loop {
        let mut message = match router_socket.recv().await {
            Ok(message) => message.into_vecdeque(),
            Err(e) => {
                eprintln!("server failed to receive message: {}", e);
                continue 'receive_messages;
            }
        };
        let message_bytes = message
            .pop_back()
            .expect("server received message without required data.");
        let message_string = unsafe { String::from_utf8_unchecked(message_bytes.bytes().to_vec()) };

        println!("server incoming message: {:?}", message_string.clone());

        match x_pub_socket
            .send(ZmqMessage::from(message_string.clone()))
            .await
        {
            Ok(()) => println!("server send message: {:?}", message_string),
            Err(e) => eprintln!("server failed to send message: {}", e),
        }
    }
}
