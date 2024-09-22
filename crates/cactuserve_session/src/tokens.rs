use base64::{prelude::BASE64_URL_SAFE, write::EncoderWriter};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct Opaque<T>(T);

#[derive(Debug)]
pub struct Token<P, O = ()> {
    payload: P,
    opaque: Opaque<O>,
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeTokenError {
    #[error(transparent)]
    Postcard(#[from] postcard::Error),
    #[error(transparent)]
    Base64(#[from] base64::EncodeSliceError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct EncodedToken<'buf> {
    payload: &'buf [u8],
    checksum: u32,
}

impl<P: Serialize, O> Token<P, O> {
    pub fn encode<'b>(&self, buf: &'b mut [u8]) -> Result<EncodedToken<'b>, EncodeTokenError> {
        let writer = EncoderWriter::new(buf, &BASE64_URL_SAFE);

        let payload = postcard::to_io(&self.payload, writer)?.finish()?;
        let checksum = crc32fast::hash(payload);

        Ok(EncodedToken { payload, checksum })
    }
}
