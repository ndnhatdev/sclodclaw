//! IPC helper functions for process runner.

use crate::core::runtime::process_ipc_codec::{decode_next_message, write_message};
use crate::core::runtime::process_ipc_messages::{RequestEnvelope, ResponseEnvelope};
use std::io::{BufRead, Write};

pub fn send_request<W: Write>(writer: &mut W, request: &RequestEnvelope) -> anyhow::Result<()> {
    write_message(writer, request)
}

pub fn read_response<R: BufRead>(reader: &mut R) -> anyhow::Result<Option<ResponseEnvelope>> {
    decode_next_message(reader)
}
