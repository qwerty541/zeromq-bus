// Rust flags
#![warn(nonstandard_style)]
#![warn(future_incompatible)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![warn(missing_debug_implementations)]
#![warn(missing_copy_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]
#![recursion_limit = "1024"]
// Clippy flags
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

#[macro_export]
macro_rules! __format_endpoint {
    ($endpoint:expr) => {
        format!("tcp://{}", $endpoint)
    };
}

lazy_static::lazy_static! {
    pub static ref __BUS_ROUTER_SOCKET_ADDR: String = format_endpoint!("0.0.0.0:56731");
    pub static ref __BUS_PUBLISHERS_SOCKET_ADDRS: Vec<String> = vec![
        format_endpoint!("0.0.0.0:56738"),
        format_endpoint!("0.0.0.0:56739"),
        format_endpoint!("0.0.0.0:56740"),
        format_endpoint!("0.0.0.0:56741"),
        format_endpoint!("0.0.0.0:56742")
    ];
}

pub const REQUESTS_COUNT_INSIDE_ONE_GROUP: usize = 25_000;
pub const ZEROMQ_ZERO_FLAG: i32 = 0;
pub const LOG_LEVEL: &str = "debug";
pub const RUST_LOG_ENVIRONMENT_VARIABLE_NAME: &str = "RUST_LOG";

mod helpers;
pub use helpers::BusPublisherData;
pub use helpers::MutexLock;

pub use __format_endpoint as format_endpoint;
pub use __BUS_PUBLISHERS_SOCKET_ADDRS as BUS_PUBLISHERS_SOCKET_ADDRS;
pub use __BUS_ROUTER_SOCKET_ADDR as BUS_ROUTER_SOCKET_ADDR;
