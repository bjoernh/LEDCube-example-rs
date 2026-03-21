use prost::Message;
use cobs::{encode_vec, decode_vec};

pub mod matrixserver {
    include!(concat!(env!("OUT_DIR"), "/matrixserver.rs"));
}

use matrixserver::MatrixServerMessage;

/// Encode a message into COBS frame with a terminating '0x00' byte
pub fn encode_message(msg: &MatrixServerMessage) -> Vec<u8> {
    let mut buf = Vec::new();
    msg.encode(&mut buf).unwrap(); // Encode protobuf
    let mut cobs_encoded = encode_vec(&buf); // Encode COBS
    cobs_encoded.push(0x00); // Add delimiter
    cobs_encoded
}

/// Decode a COBS frame into a protobuf message
pub fn decode_message(frame: &[u8]) -> Result<MatrixServerMessage, prost::DecodeError> {
    // remove zero byte if present at the end
    let frame = if frame.last() == Some(&0) {
        &frame[..frame.len() - 1]
    } else {
        frame
    };
    
    match decode_vec(frame) {
        Ok(decoded) => MatrixServerMessage::decode(&decoded[..]),
        Err(_) => Err(prost::DecodeError::new("COBS decode failed")),
    }
}
