use schemafy::schemafy;
use serde::Deserialize;
use serde::Serialize;

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
