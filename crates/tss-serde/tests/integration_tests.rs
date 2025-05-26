use tss_serde::{TssDeserialize, TssSerialize};

#[derive(TssSerialize, TssDeserialize, Debug, PartialEq)]
struct TpmGetRandomCommand {
    tag: u16,
    length: u32,
    command_code: u32,
    bytes_requested: u16,
}

#[derive(TssSerialize, TssDeserialize, Debug, PartialEq)]
struct TpmResponse {
    tag: u16,
    length: u32,
    response_code: u32,
    data: [u8; 4],
}

#[test]
fn test_serialize_command() {
    let cmd = TpmGetRandomCommand {
        tag: 0x8001,
        length: 12,
        command_code: 0x017B,
        bytes_requested: 32,
    };

    let bytes = cmd.to_tss_bytes();
    let expected = vec![
        0x80, 0x01, // tag
        0x00, 0x00, 0x00, 0x0C, // length
        0x00, 0x00, 0x01, 0x7B, // command_code
        0x00, 0x20, // bytes_requested
    ];

    assert_eq!(bytes, expected);
}

#[test]
fn test_deserialize_response() {
    let response_bytes = vec![
        0x80, 0x01, // tag
        0x00, 0x00, 0x00, 0x0E, // length
        0x00, 0x00, 0x00, 0x00, // response_code (success)
        0xAB, 0xCD, 0xEF, 0x12, // data
    ];

    let response = TpmResponse::from_tss_bytes(&response_bytes).unwrap();

    assert_eq!(response.tag, 0x8001);
    assert_eq!(response.length, 14);
    assert_eq!(response.response_code, 0);
    assert_eq!(response.data, [0xAB, 0xCD, 0xEF, 0x12]);
}

#[test]
fn test_roundtrip() {
    let original = TpmGetRandomCommand {
        tag: 0x8001,
        length: 12,
        command_code: 0x017B,
        bytes_requested: 32,
    };

    let bytes = original.to_tss_bytes();
    let decoded = TpmGetRandomCommand::from_tss_bytes(&bytes).unwrap();

    assert_eq!(original, decoded);
}
