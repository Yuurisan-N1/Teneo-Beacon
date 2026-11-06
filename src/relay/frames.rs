pub const OP_CONNECT: u8 = 0x01;
pub const OP_DATA: u8 = 0x02;
pub const OP_CLOSE: u8 = 0x04;
pub const OP_CONTROL: u8 = 0x06;

pub struct Frame {
    pub stream: u32,
    pub opcode: u8,
    pub payload: Vec<u8>,
}

pub fn encode(stream: u32, opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.extend_from_slice(&stream.to_be_bytes());
    out.push(opcode);
    out.extend_from_slice(payload);
    out
}

pub fn decode(buf: &[u8]) -> Option<Frame> {
    if buf.len() < 5 {
        return None;
    }
    let stream = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
    Some(Frame {
        stream,
        opcode: buf[4],
        payload: buf[5..].to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_keeps_the_stream_and_the_opcode() {
        let raw = encode(7, OP_DATA, b"hello");
        let frame = decode(&raw).unwrap();
        assert_eq!(frame.stream, 7);
        assert_eq!(frame.opcode, OP_DATA);
        assert_eq!(frame.payload, b"hello");
    }

    #[test]
    fn short_buffers_are_refused() {
        assert!(decode(&[0, 0, 0, 1]).is_none());
    }
}
