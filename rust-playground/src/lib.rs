#[macro_export]
macro_rules! __format_endpoint {
    ($endpoint:expr) => {
        format!("tcp://{}", $endpoint)
    };
}

lazy_static::lazy_static! {
    pub static ref __SERVER_ROUTER_SOCKET_ADDR: String = format_endpoint!("0.0.0.0:56731");
    pub static ref __SERVER_PUBLISHER_SOCKET_ADDRS: Vec<String> = vec![
        format_endpoint!("0.0.0.0:56738"),
        format_endpoint!("0.0.0.0:56739"),
        format_endpoint!("0.0.0.0:56740"),
        format_endpoint!("0.0.0.0:56741"),
        format_endpoint!("0.0.0.0:56742")
    ];
}

pub const ZEROMQ_MESSAGE_SEND_TIMEOUT_MILLIS: u64 = 1_000;
pub const MESSAGE_CONTENT_LENGTH: usize = 16;
pub const COUNT_OF_ZEROMQ_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT: usize = 10_000;
pub const COUNT_OF_ZEROMQ_FFI_MESSAGES_THAT_SHOULD_BE_SENT_EVERY_TIMEOUT: usize = 200_000;
pub const ZEROMQ_FFI_ZERO_FLAG: i32 = 0;

mod helpers;
pub use helpers::format_zmq_message;
pub use helpers::MessageKind;
pub use helpers::NoRequiredDataError;
pub use helpers::RequestData;

pub use __format_endpoint as format_endpoint;
pub use __SERVER_PUBLISHER_SOCKET_ADDRS as SERVER_PUBLISHER_SOCKET_ADDRS;
pub use __SERVER_ROUTER_SOCKET_ADDR as SERVER_ROUTER_SOCKET_ADDR;
