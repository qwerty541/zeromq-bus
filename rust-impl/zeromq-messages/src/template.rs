use crate::kind::ZeromqMessageKind;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::fmt;

pub trait ZeromqMessageTrait<'de>:
    Clone + PartialEq + fmt::Debug + Serialize + Deserialize<'de> + Sized
{
    fn kind() -> ZeromqMessageKind;

    fn serialize(self) -> Value {
        json!(self)
    }

    fn serialize_cloned(&self) -> Value {
        json!(self.clone())
    }
}
