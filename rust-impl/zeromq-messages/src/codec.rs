use crate::kind::ZeromqMessageKind;
use crate::template::ZeromqMessageTrait;
use bytes::Buf;
use bytes::BufMut;
use num_enum::TryFromPrimitiveError;
use serde::ser::Error;
use std::convert::TryFrom;
use std::string::ToString;
use uuid::Uuid;

//-----------------------------------------------------------------------------------------
// Errors
//-----------------------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum MessageDecodeError {
    #[error("Unexpected message kind received")]
    UnexpectedZeromqMessageKind(#[from] TryFromPrimitiveError<ZeromqMessageKind>),

    #[error("Failed to parse json into struct")]
    CantParseJson(#[source] serde_json::Error),
}

impl Clone for MessageDecodeError {
    fn clone(&self) -> Self {
        match self {
            Self::UnexpectedZeromqMessageKind(TryFromPrimitiveError { number }) => {
                Self::UnexpectedZeromqMessageKind(TryFromPrimitiveError { number: *number })
            }
            Self::CantParseJson(error) => {
                Self::CantParseJson(serde_json::Error::custom(error.to_string()))
            }
        }
    }
}

impl PartialEq for MessageDecodeError {
    #[allow(clippy::match_wildcard_for_single_variants)]
    fn eq(&self, other: &MessageDecodeError) -> bool {
        match self {
            Self::UnexpectedZeromqMessageKind(error) => match other {
                Self::UnexpectedZeromqMessageKind(other_error) => error == other_error,
                _ => false,
            },
            Self::CantParseJson(error) => match other {
                Self::CantParseJson(other_error) => {
                    error.to_string() == other_error.to_string()
                }
                _ => false,
            },
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MessageEncodeError {
    #[error("Failed to create json from message payload")]
    CantCreateJsonFromMessagePayload(#[source] serde_json::Error),
}

impl Clone for MessageEncodeError {
    fn clone(&self) -> Self {
        match self {
            Self::CantCreateJsonFromMessagePayload(error) => {
                Self::CantCreateJsonFromMessagePayload(serde_json::Error::custom(
                    error.to_string(),
                ))
            }
        }
    }
}

impl PartialEq for MessageEncodeError {
    fn eq(&self, other: &MessageEncodeError) -> bool {
        match self {
            Self::CantCreateJsonFromMessagePayload(error) => match other {
                Self::CantCreateJsonFromMessagePayload(other_error) => {
                    error.to_string() == other_error.to_string()
                }
            },
        }
    }
}

//-----------------------------------------------------------------------------------------
// Encode
//-----------------------------------------------------------------------------------------

pub fn encode_message<'de, P: ZeromqMessageTrait<'de>>(
    uuid: Uuid,
    payload: P,
) -> Result<Vec<u8>, MessageEncodeError> {
    let mut output_message_bytes: Vec<u8> = Vec::default();

    output_message_bytes.put_u32(<P as ZeromqMessageTrait<'de>>::kind() as u32);

    output_message_bytes.put_u128(uuid.as_u128());

    let payload_string = serde_json::to_value(payload)
        .map_err(MessageEncodeError::CantCreateJsonFromMessagePayload)?
        .to_string();

    for byte in payload_string.as_bytes() {
        output_message_bytes.put_u8(*byte);
    }

    Ok(output_message_bytes)
}

//-----------------------------------------------------------------------------------------
// Decode
//-----------------------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)]
pub fn decode_message_kind(
    message_bytes: Vec<u8>,
) -> Result<(ZeromqMessageKind, Vec<u8>), MessageDecodeError> {
    let mut message_bytes_slice = message_bytes.as_slice();

    // Grab message kind.
    let kind = ZeromqMessageKind::try_from(message_bytes_slice.get_u32())
        .map_err(MessageDecodeError::UnexpectedZeromqMessageKind)?;

    Ok((kind, message_bytes_slice.chunk().to_vec()))
}

#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn decode_message_uuid(message_bytes_without_kind: Vec<u8>) -> (Uuid, Vec<u8>) {
    let mut message_bytes_slice_without_kind = message_bytes_without_kind.as_slice();
    let uuid_bytes = message_bytes_slice_without_kind.get_u128();
    (
        Uuid::from_u128(uuid_bytes),
        message_bytes_slice_without_kind.chunk().to_vec(),
    )
}

pub fn decode_message_payload<'de, T: ZeromqMessageTrait<'de>>(
    message_bytes_without_kind_and_uuid: &'de [u8],
) -> Result<T, MessageDecodeError> {
    serde_json::from_slice(message_bytes_without_kind_and_uuid)
        .map_err(MessageDecodeError::CantParseJson)
}

//-----------------------------------------------------------------------------------------
// Tests
//-----------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::codec::decode_message_kind;
    use crate::codec::decode_message_payload;
    use crate::codec::decode_message_uuid;
    use crate::codec::encode_message;
    use crate::codec::MessageDecodeError;
    use crate::codec::MessageEncodeError;
    use crate::kind::ZeromqMessageKind;
    use crate::messages::ValueMultiplicationRequest;
    use crate::template::ZeromqMessageTrait;
    use num_enum::TryFromPrimitiveError;
    use std::convert::From;
    use uuid::Uuid;
    use zmq::Message;

    fn get_json_data_error() -> serde_json::Error {
        let maybe_json_error = serde_json::from_str::<'_, ()>("{[[[]]][[]]}");
        let json_error = maybe_json_error.unwrap_err();
        assert!(json_error.is_data());
        json_error
    }

    fn get_json_syntax_error() -> serde_json::Error {
        let maybe_json_error = serde_json::from_str::<'_, ()>("////////////\\");
        let json_error = maybe_json_error.unwrap_err();
        assert!(json_error.is_syntax());
        json_error
    }

    #[test]
    fn basics() {
        let payload = ValueMultiplicationRequest {
            value: 5,
            multiplier: 5,
        };
        let uuid = Uuid::new_v4();

        let encoded_zeromq_message = Message::from(
            encode_message(uuid, payload.clone()).expect("failed to encode message"),
        );

        let (decoded_kind, remaining_bytes) =
            decode_message_kind(encoded_zeromq_message.to_vec())
                .expect("failed to decode message kind");
        assert_eq!(
            <ValueMultiplicationRequest as ZeromqMessageTrait>::kind(),
            decoded_kind
        );

        let (decoded_uuid, remaining_bytes) = decode_message_uuid(remaining_bytes);
        assert_eq!(uuid, decoded_uuid);

        let decoded_payload = decode_message_payload(remaining_bytes.as_slice())
            .expect("failed to decode message payload");
        assert_eq!(payload, decoded_payload);
    }

    #[test]
    fn encode_error_eq() {
        assert_eq!(
            MessageEncodeError::CantCreateJsonFromMessagePayload(get_json_syntax_error()),
            MessageEncodeError::CantCreateJsonFromMessagePayload(get_json_syntax_error())
        );

        assert_ne!(
            MessageEncodeError::CantCreateJsonFromMessagePayload(get_json_syntax_error()),
            MessageEncodeError::CantCreateJsonFromMessagePayload(get_json_data_error())
        );
    }

    #[test]
    fn decode_error_eq() {
        assert_eq!(
            MessageDecodeError::UnexpectedZeromqMessageKind(TryFromPrimitiveError {
                number: ZeromqMessageKind::ValueMultiplicationRequest as u32
            }),
            MessageDecodeError::UnexpectedZeromqMessageKind(TryFromPrimitiveError {
                number: ZeromqMessageKind::ValueMultiplicationRequest as u32
            })
        );

        assert_eq!(
            MessageDecodeError::CantParseJson(get_json_data_error()),
            MessageDecodeError::CantParseJson(get_json_data_error())
        );

        assert_ne!(
            MessageDecodeError::UnexpectedZeromqMessageKind(TryFromPrimitiveError {
                number: ZeromqMessageKind::ValueMultiplicationResponse as u32
            }),
            MessageDecodeError::UnexpectedZeromqMessageKind(TryFromPrimitiveError {
                number: ZeromqMessageKind::ValueMultiplicationRequest as u32
            })
        );

        assert_ne!(
            MessageDecodeError::CantParseJson(get_json_data_error()),
            MessageDecodeError::CantParseJson(get_json_syntax_error())
        );

        assert_ne!(
            MessageDecodeError::UnexpectedZeromqMessageKind(TryFromPrimitiveError {
                number: ZeromqMessageKind::ValueMultiplicationResponse as u32
            }),
            MessageDecodeError::CantParseJson(get_json_syntax_error())
        );
    }

    #[test]
    fn errors_clone() {
        let encode_error =
            MessageEncodeError::CantCreateJsonFromMessagePayload(get_json_syntax_error());
        assert_eq!(encode_error, encode_error.clone());

        let decode_error_json = MessageDecodeError::CantParseJson(get_json_data_error());
        assert_eq!(decode_error_json, decode_error_json.clone());

        let decode_error_kind =
            MessageDecodeError::UnexpectedZeromqMessageKind(TryFromPrimitiveError {
                number: ZeromqMessageKind::ValueMultiplicationResponse as u32,
            });
        assert_eq!(decode_error_kind, decode_error_kind.clone());
    }
}
