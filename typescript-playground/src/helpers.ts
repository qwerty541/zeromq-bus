export const send_messages_timeout_millis: number = 1000;
export const count_of_messages_that_should_be_sended_every_timeout: number = 200_000;
export const message_content_length: number = 16;
export const server_router_socket_addr: string = "0.0.0.0:56731";
export const server_publisher_socket_addrs: Array<string> = [
    "0.0.0.0:56738",
    "0.0.0.0:56739",
    "0.0.0.0:56740",
    "0.0.0.0:56741",
    "0.0.0.0:56742",
];

export function sleep(millis: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, millis));
}

export function format_endpoint(endpoint: string): string {
    return `tcp://${endpoint}`;
}
