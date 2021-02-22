#![recursion_limit = "1024"]

use futures::select;
use futures::FutureExt;
use rust_playground::format_zmq_message;
use rust_playground::SERVER_PUBLISHER_SOCKET_ADDR;
use rust_playground::SERVER_ROUTER_SOCKET_ADDR;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use zeromq::PubSocket;
use zeromq::RouterSocket;
use zeromq::Socket;
use zeromq::SocketRecv;
use zeromq::SocketSend;
use zeromq::ZmqMessage;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // Init server router socket.
    let mut router_socket = RouterSocket::new();

    router_socket
        .bind(SERVER_ROUTER_SOCKET_ADDR.as_str())
        .await
        .expect("failed to bind server router socket");

    println!("init server router socket");

    // Init server publisher socket.
    let mut x_pub_socket = PubSocket::new();

    x_pub_socket
        .bind(SERVER_PUBLISHER_SOCKET_ADDR.as_str())
        .await
        .expect("failed to bind server publisher socket");

    println!("init server publisher socket");

    let x_pub_socket = Arc::new(Mutex::new(x_pub_socket));

    // Init channel for errored messages.
    let (errored_messages_channel_sender, mut errored_messages_channel_receiver) =
        mpsc::unbounded_channel::<ZmqMessage>();

    println!("init channel for errored messages");

    println!("running messages processing loop");

    // Messages processing loop.
    'receive_messages: loop {
        let mut is_errored_message = false;
        let message = select! {
            maybe_message = router_socket.recv().fuse() => {
                match maybe_message {
                    Ok(message) => message,
                    Err(e) => {
                        eprintln!("router socket failed to receive message: {}", e);
                        continue 'receive_messages;
                    },
                }
            },
            maybe_errored_message = errored_messages_channel_receiver.recv().fuse() => {
                match maybe_errored_message {
                    Some(message) => {
                        is_errored_message = true;
                        message
                    },
                    None => {
                        panic!("errored messages sender dropped");
                    }
                }
            }
        };

        let cloned_errored_messages_channel_sender = errored_messages_channel_sender.clone();
        let cloned_x_pub_socket = Arc::clone(&x_pub_socket);
        let _ = tokio::spawn(async move {
            let message_string = unsafe {
                format_zmq_message(message.clone())
                    .expect("server received message without required data.")
            };

            if !is_errored_message {
                println!("server incoming message: {:?}", message_string.clone());
            }

            match cloned_x_pub_socket.lock().await.send(message.clone()).await {
                Ok(()) => println!("server send message: {:?}", message_string),
                Err(e) => {
                    eprintln!("server failed to send message: {}", e);
                    cloned_errored_messages_channel_sender
                        .send(message)
                        .expect("errored message channel receiver droppped");
                }
            }
        });
    }
}
