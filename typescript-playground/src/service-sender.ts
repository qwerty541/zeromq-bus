import * as zeromq from "zeromq";
import {
    sleep,
    message_content_length,
    send_commands_timeout_millis,
    RequestData,
    format_endpoint,
    server_router_socket_addr,
} from "./helpers";

export async function run_sender(server_router_socket_addr: string) {
    const sender = new zeromq.Dealer();

    console.log("init sender");

    sender.connect(format_endpoint(server_router_socket_addr));

    console.log("sender connected");

    for (;;) {
        let message_string = JSON.stringify(
            new RequestData(Math.random().toString(message_content_length)),
        );
        await sender.send(message_string);
        console.log(`Send message: ${message_string}`);

        await sleep(send_commands_timeout_millis);
    }
}

run_sender(server_router_socket_addr).catch((e) => console.error(e));
