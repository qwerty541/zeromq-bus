use crate::kind::ZeromqMessageKind;
use crate::template::ZeromqMessageTrait;
use schemafy::schemafy;
use serde::Deserialize;
use serde::Serialize;
use zeromq_messages_gen::generate_zeromq_messages_structs;

generate_zeromq_messages_structs!();
