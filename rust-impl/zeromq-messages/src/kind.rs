use num_enum::TryFromPrimitive;
use zeromq_messages_gen::generate_zeromq_messages_kinds_enum;

generate_zeromq_messages_kinds_enum!();
