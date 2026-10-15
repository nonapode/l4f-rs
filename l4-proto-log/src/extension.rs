use l4_ipc_core::protocol::Speaks;
use l4_ipc_core::{Error, Timeout};
use l4_sys::Utcb;

use super::Protocol;
use super::protocol::Bytes;
use super::operation as op;

/// Convenience methods for handles that speak the log protocol.
pub trait Log: Speaks<Protocol> {
    /// Write `bytes` (at most [`Protocol::WRITE_SIZE`]) using the given UTCB.
    fn write_with(&self, utcb: &Utcb, bytes: &[u8]) -> Result<(), Error> {
        self.invoke_with::<op::Write>(utcb, Bytes(bytes), Timeout::NEVER)
    }

    /// Write `text` in as many messages as needed using the given UTCB.
    fn print_with(&self, utcb: &Utcb, text: &str) -> Result<(), Error> {
        for chunk in text.as_bytes().chunks(Protocol::WRITE_SIZE) {
            self.write_with(utcb, chunk)?;
        }
        Ok(())
    }

    /// Write `bytes` (at most [`Protocol::WRITE_SIZE`]) using the current UTCB.
    fn write(&self, bytes: &[u8]) -> Result<(), Error> {
        self.write_with(Utcb::current(), bytes)
    }

    /// Write `text` in as many messages as needed using the current UTCB.
    fn print(&self, text: &str) -> Result<(), Error> {
        self.print_with(Utcb::current(), text)
    }

    /// A buffered [`core::fmt::Write`] adapter for this handle.
    fn console(self) -> Console<Self>
    where
        Self: Sized,
    {
        Console::new(self)
    }
}

impl<T: Speaks<Protocol>> Log for T {}

/// Line buffer that hands out chunks of at most [`Protocol::WRITE_SIZE`] bytes.
struct Buffer {
    bytes: [u8; Protocol::WRITE_SIZE],
    len: usize,
}

impl Buffer {
    const fn new() -> Self {
        Buffer {
            bytes: [0; Protocol::WRITE_SIZE],
            len: 0,
        }
    }

    /// Append `text`; pass full buffers, and everything once `text`
    /// contains a newline, to `sink`.
    fn push(
        &mut self,
        text: &str,
        sink: &mut dyn FnMut(&[u8]) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let mut rest = text.as_bytes();
        while !rest.is_empty() {
            let n = (Protocol::WRITE_SIZE - self.len).min(rest.len());
            self.bytes[self.len..self.len + n].copy_from_slice(&rest[..n]);
            self.len += n;
            rest = &rest[n..];
            if self.len == Protocol::WRITE_SIZE {
                self.flush(sink)?;
            }
        }
        if text.as_bytes().contains(&b'\n') {
            self.flush(sink)?;
        }
        Ok(())
    }

    fn flush(&mut self, sink: &mut dyn FnMut(&[u8]) -> Result<(), Error>) -> Result<(), Error> {
        let len = core::mem::take(&mut self.len);
        if len == 0 {
            return Ok(());
        }
        sink(&self.bytes[..len])
    }
}

/// Buffered [`core::fmt::Write`] adapter for a handle that speaks the log
/// protocol.
///
/// `write!` calls `write_str` once per literal piece and per argument; the
/// console collects them and only sends a message when the buffer is full,
/// when written text contains a newline, on [`flush`](Console::flush) and
/// on drop.
pub struct Console<H: Speaks<Protocol>> {
    log: H,
    buffer: Buffer,
    error: Option<Error>,
}

impl<H: Speaks<Protocol>> Console<H> {
    /// Create a console writing to `log`.
    pub const fn new(log: H) -> Self {
        Console {
            log,
            buffer: Buffer::new(),
            error: None,
        }
    }

    /// Send any buffered text.
    pub fn flush(&mut self) -> Result<(), Error> {
        let log = self.log;
        self.buffer.flush(&mut |bytes| log.write(bytes))
    }

    /// The first IPC error that made a write fail (`fmt::Error` carries
    /// no information).
    pub fn take_error(&mut self) -> Option<Error> {
        self.error.take()
    }
}

impl<H: Speaks<Protocol>> core::fmt::Write for Console<H> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let log = self.log;
        self.buffer
            .push(s, &mut |bytes| log.write(bytes))
            .map_err(|e| {
                self.error.get_or_insert(e);
                core::fmt::Error
            })
    }
}

impl<H: Speaks<Protocol>> Drop for Console<H> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l4_ipc_core::protocol::encode_request;
    use l4_sys::{umword, MR_COUNT};
    use l4_ipc_core::message::TagFlags;

    #[test]
    fn write_request_matches_vcon_wire_format() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<op::Write>(&utcb, &Bytes(b"hello")).unwrap();

        assert_eq!(tag.label(), -13);
        assert_eq!(tag.words(), 3); // opcode, size, one word of text
        assert_eq!(tag.items(), 0);
        assert_eq!(tag.flags(), TagFlags::SCHEDULE);

        assert_eq!(utcb.mr(0), 0); // L4_VCON_WRITE_OP
        assert_eq!(utcb.mr(1), 5);
        let mut text = [0u8; size_of::<umword>()];
        text[..5].copy_from_slice(b"hello");
        assert_eq!(utcb.mr(2), umword::from_ne_bytes(text));
    }

    #[test]
    fn maximum_write_fits_exactly() {
        let utcb = Utcb::zeroed();
        let max = [b'x'; Protocol::WRITE_SIZE];
        let tag = encode_request::<op::Write>(&utcb, &Bytes(&max)).unwrap();
        assert_eq!(tag.words(), MR_COUNT);

        let too_long = [b'x'; Protocol::WRITE_SIZE + 1];
        assert_eq!(
            encode_request::<op::Write>(&utcb, &Bytes(&too_long)),
            Err(Error::Overflow)
        );
    }

    fn collect(buffer: &mut Buffer, text: &str, out: &mut std::vec::Vec<std::vec::Vec<u8>>) {
        buffer
            .push(text, &mut |bytes| {
                out.push(bytes.to_vec());
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn buffer_holds_text_until_newline() {
        let mut b = Buffer::new();
        let mut out = std::vec::Vec::new();
        collect(&mut b, "ab", &mut out);
        collect(&mut b, "c", &mut out);
        assert!(out.is_empty());
        collect(&mut b, "d\ne", &mut out);
        assert_eq!(out, [b"abcd\ne".to_vec()]);
    }

    #[test]
    fn buffer_flushes_full_chunks() {
        let mut b = Buffer::new();
        let mut out = std::vec::Vec::new();
        let text = "x".repeat(2 * Protocol::WRITE_SIZE + 3);
        collect(&mut b, &text, &mut out);
        assert_eq!(out.iter().map(|c| c.len()).collect::<std::vec::Vec<_>>(), [Protocol::WRITE_SIZE, Protocol::WRITE_SIZE]);
        let mut rest = std::vec::Vec::new();
        b.flush(&mut |bytes| {
            rest.extend_from_slice(bytes);
            Ok(())
        })
        .unwrap();
        assert_eq!(rest, b"xxx");
    }

    #[test]
    fn buffer_error_is_reported_and_buffer_cleared() {
        let mut b = Buffer::new();
        let r = b.push("hi\n", &mut |_| Err(Error::Ipc(1)));
        assert_eq!(r, Err(Error::Ipc(1)));
        let mut out = std::vec::Vec::new();
        collect(&mut b, "ok\n", &mut out);
        assert_eq!(out, [b"ok\n".to_vec()]);
    }
}
