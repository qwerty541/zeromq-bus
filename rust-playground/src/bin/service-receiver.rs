use rust_playground::format_zmq_message;
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

    'receive_messages: loop {
        let message = match rep_socket.recv().await {
            Ok(message) => message,
            Err(e) => {
                eprintln!("failed to receive message: {}", e);
                continue 'receive_messages;
            }
        };
        let message_string = unsafe {
            format_zmq_message(message).expect("server received message without required data.")
        };

        println!("incoming message: {:?}", message_string);
    }
}
