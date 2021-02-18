import * as zeromq from "zeromq";
import { format_endpoint, server_publisher_socket_addr } from "./helpers";

async function run_receiver(server_publisher_socket_addr: string) {
    const receiver = new zeromq.Subscriber();

    console.log("init receiver");

    receiver.connect(format_endpoint(server_publisher_socket_addr));
    receiver.subscribe();

    console.log("receiver connected");

    for (;;) {
        const message_string = (await receiver.receive()).toString();
        console.log(`Received message: ${message_string}`);
    }
}

run_receiver(server_publisher_socket_addr).catch(e => console.error(e));
