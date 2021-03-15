import * as zeromq from "zeromq";
import {
    sleep,
    message_content_length,
    send_messages_timeout_millis,
    format_endpoint,
    server_router_socket_addr,
    count_of_messages_that_should_be_sended_every_timeout,
} from "./helpers";
import { RequestData } from "./types/requestData";

async function run_sender(server_router_socket_addr: string) {
    const sender = new zeromq.Dealer();

    console.log("init sender");

    sender.connect(format_endpoint(server_router_socket_addr));

    console.log("sender connected");

    let total_sended = 0;
    for (;;) {
        let message_data: RequestData = {
            content: [...Array(message_content_length)].map(()=>(~~(Math.random()*36)).toString(36)).join(''),
        };
        let message_string = JSON.stringify(message_data);

        for (let i = 0; i < count_of_messages_that_should_be_sended_every_timeout; i++) {
            await sender.send(message_string);
        }

        total_sended += count_of_messages_that_should_be_sended_every_timeout;

        let date = new Date();
        console.log(
            `${date.getHours()}:${date.getMinutes()}:${date.getSeconds()} | total sended ${total_sended} messages`,
        );

        await sleep(send_messages_timeout_millis);
    }
}

run_sender(server_router_socket_addr).catch((e) => console.error(e));
