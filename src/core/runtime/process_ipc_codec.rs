//! Content-Length framed IPC codec.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::{BufRead, Write};

const CONTENT_LENGTH: &str = "Content-Length:";

pub fn encode_message<T: Serialize>(message: &T) -> anyhow::Result<Vec<u8>> {
    let payload = serde_json::to_vec(message)?;
    let mut framed = Vec::with_capacity(payload.len() + 64);
    framed.extend_from_slice(format!("{CONTENT_LENGTH} {}\r\n\r\n", payload.len()).as_bytes());
    framed.extend_from_slice(&payload);
    Ok(framed)
}

pub fn write_message<W: Write, T: Serialize>(writer: &mut W, message: &T) -> anyhow::Result<()> {
    let framed = encode_message(message)?;
    writer.write_all(&framed)?;
    writer.flush()?;
    Ok(())
}

pub fn decode_next_message<R: BufRead, T: DeserializeOwned>(
    reader: &mut R,
) -> anyhow::Result<Option<T>> {
    let mut header_line = String::new();
    let read = reader.read_line(&mut header_line)?;
    if read == 0 {
        return Ok(None);
    }

    if !header_line.starts_with(CONTENT_LENGTH) {
        anyhow::bail!("ipc.invalid_frame: missing Content-Length header");
    }

    let len = header_line[CONTENT_LENGTH.len()..]
        .trim()
        .parse::<usize>()?;

    let mut separator = String::new();
    let read_separator = reader.read_line(&mut separator)?;
    if read_separator == 0 || separator != "\r\n" {
        anyhow::bail!("ipc.invalid_frame: missing header separator");
    }

    let mut payload = vec![0_u8; len];
    reader.read_exact(&mut payload)?;
    let decoded = serde_json::from_slice::<T>(&payload)?;
    Ok(Some(decoded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_framed_json() {
        let original =
            serde_json::json!({"type":"request","id":"req-1","method":"invoke","payload":{}});
        let bytes = encode_message(&original).expect("encode");
        let mut reader = std::io::BufReader::new(std::io::Cursor::new(bytes));
        let decoded = decode_next_message::<_, serde_json::Value>(&mut reader)
            .expect("decode")
            .expect("message");
        assert_eq!(decoded, original);
    }
}
