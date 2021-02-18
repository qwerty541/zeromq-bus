#[macro_export]
macro_rules! __format_endpoint {
    ($endpoint:expr) => {
        format!("tcp://{}", $endpoint)
    };
}

lazy_static::lazy_static! {
    pub static ref __SERVER_ROUTER_SOCKET_ADDR: String = format_endpoint!("0.0.0.0:56737");
    pub static ref __SERVER_PUBLISHER_SOCKET_ADDR: String = format_endpoint!("0.0.0.0:56738");
}

pub const COMMANDS_SEND_TIMEOUT_MILLIS: u64 = 1000;
pub const MESSAGE_CONTENT_LENGTH: usize = 16;
pub const COUNT_OF_COMMANDS_THAT_SHOULD_BE_SENT_EVERY_TIMEIUT: usize = 1;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequestData {
    pub content: String,
}

pub use __format_endpoint as format_endpoint;
pub use __SERVER_PUBLISHER_SOCKET_ADDR as SERVER_PUBLISHER_SOCKET_ADDR;
pub use __SERVER_ROUTER_SOCKET_ADDR as SERVER_ROUTER_SOCKET_ADDR;
