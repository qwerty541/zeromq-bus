use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use zeromq_messages_gen::generate_zeromq_messages_kinds_enum;

generate_zeromq_messages_kinds_enum!();

#[test]
fn basic() {
    assert_eq!(
        Ok(ZeromqMessageKind::ValueMultiplicationRequest),
        <ZeromqMessageKind as TryFrom<u32>>::try_from(1_u32)
    );
}

#[test]
#[should_panic]
fn error() {
    <ZeromqMessageKind as TryFrom<u32>>::try_from(0_u32).unwrap();
}
