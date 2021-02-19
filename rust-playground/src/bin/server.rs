use bytes::Buf;
use rust_playground::SERVER_PUBLISHER_SOCKET_ADDR;
use rust_playground::SERVER_ROUTER_SOCKET_ADDR;
use zeromq::PubSocket;
use zeromq::RouterSocket;
use zeromq::Socket;
use zeromq::SocketRecv;
use zeromq::SocketSend;
use zeromq::ZmqMessage;

#[tokio::main(flavor = "current_thread")]
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

    // Messages processing loop.
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
