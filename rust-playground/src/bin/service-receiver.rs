use bytes::Buf;
use rust_playground::SERVER_PUBLISHER_SOCKET_ADDR;
use zeromq::Socket;
use zeromq::SocketRecv;
use zeromq::SubSocket;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut rep_socket = SubSocket::new();

    println!("init receiver");

    rep_socket
        .connect(SERVER_PUBLISHER_SOCKET_ADDR.as_str())
        .await
        .expect("failed to connect to server dealer socket.");

    rep_socket
        .subscribe("")
        .await
        .expect("failed to subscriner to server dealer");

    println!("receiver connected");

    'client_receive_messages: loop {
        let mut message = match rep_socket.recv().await {
            Ok(message) => message.into_vecdeque(),
            Err(e) => {
                eprintln!("failed to receive message: {}", e);
                continue 'client_receive_messages;
            }
        };
        let message_bytes = message
            .pop_back()
            .expect("rep socket received message without required data.");
        let message_string = unsafe { String::from_utf8_unchecked(message_bytes.bytes().to_vec()) };

        println!("incoming message: {:?}", message_string);
    }
}
