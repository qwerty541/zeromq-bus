export const send_commands_timeout_millis: number = 1000;
export const message_content_length: number = 16;
export const server_router_socket_addr: string = "0.0.0.0:56731";
export const server_publisher_socket_addr: string = "0.0.0.0:56738";

export function sleep(millis: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, millis));
}

export function format_endpoint(endpoint: string): string {
    return `tcp://${endpoint}`;
}
