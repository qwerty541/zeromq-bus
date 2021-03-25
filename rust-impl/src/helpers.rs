use bytes::Buf;
use schemafy::schemafy;
use serde::Deserialize;
use serde::Serialize;
use zeromq::ZmqMessage;

#[derive(Debug, Clone, Copy)]
pub enum MessageKind {
    Incoming,
    Errored,
}

impl Default for MessageKind {
    fn default() -> Self {
        Self::Incoming
    }
}

schemafy!(
    root: RequestData
    "../shared/schemas/request-data.schema.json"
);

#[derive(Debug, Clone, Copy)]
pub struct NoRequiredDataError;

#[allow(clippy::missing_safety_doc)]
pub unsafe fn format_zmq_message(message: ZmqMessage) -> Result<String, NoRequiredDataError> {
    let message_bytes = match message.into_vecdeque().pop_back() {
        Some(message_bytes) => message_bytes,
        None => return Err(NoRequiredDataError),
    };

    Ok(String::from_utf8_unchecked(message_bytes.bytes().to_vec()))
}
