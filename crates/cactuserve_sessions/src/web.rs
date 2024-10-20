use std::marker::PhantomData;

use base64::{
    prelude::{Engine as _, BASE64_URL_SAFE},
    DecodeSliceError as E,
};
use chacha20::{
    cipher::{KeyIvInit, StreamCipher},
    ChaCha20,
};
use ring::signature::{Ed25519KeyPair, UnparsedPublicKey};

use crate::{TokenReader, TokenWriter};

pub trait KeyProvider {
    type Bytes: AsRef<[u8]>;

    fn public_key() -> UnparsedPublicKey<Self::Bytes>;
}

const AUTH_HEADER_SCHEME: &str = "Bearer";
const AUTH_HEADER_NAME: &str = "Authorization";

const KEY: [u8; 32] = [0x42; 32];
const NONCE: [u8; 12] = [0x24; 12];

pub struct Auth<R: TokenReader, K> {
    token: R::Token,
    _key_provider: PhantomData<K>,
}

impl<R: TokenReader, K> Auth<R, K> {
    pub(crate) const fn new(token: R::Token) -> Self {
        Self {
            token,
            _key_provider: PhantomData,
        }
    }

    pub const fn token(&self) -> &R::Token {
        &self.token
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum DecodeTokenError<E> {
    #[error(transparent)]
    InvalidBase64(#[from] base64::DecodeError),
    #[error("the token signature is invalid")]
    InalidSignature(#[from] ring::error::Unspecified),
    #[error(transparent)]
    Other(E),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum EncodeTokenError<E> {
    #[error(transparent)]
    Other(E),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthRequestError<E> {
    #[error("no 'Authorization' header or session cookie was included in the request")]
    NoSession,
    #[error("the 'Authorization' header value is malformed")]
    InvalidHeaderFormat,
    #[error("the only supported authorization scheme is 'Bearer'")]
    UnsupportedAuthScheme,
    #[error(transparent)]
    BadToken(#[from] DecodeTokenError<E>),
}

fn from_str<R: TokenReader, B: AsRef<[u8]>>(
    reader: &R,
    s: &str,
    public_key: &UnparsedPublicKey<B>,
) -> Result<R::Token, DecodeTokenError<R::Error>> {
    let bufsize = base64::decoded_len_estimate(s.len());
    let mut buf = vec![0u8; bufsize];

    let decoded_token_size =
        BASE64_URL_SAFE
            .decode_slice(s, &mut buf)
            .map_err(|err| match err {
                E::DecodeError(err) => err,
                E::OutputSliceTooSmall => unreachable!(),
            })?;

    let mut decoded_token = &mut buf[..decoded_token_size];
    let visible_size = u16::from_le_bytes([decoded_token[0], decoded_token[1]]);
    let opaque_size = u16::from_le_bytes([decoded_token[2], decoded_token[3]]);
    decoded_token = &mut decoded_token[4..];

    dbg!(visible_size);
    dbg!(opaque_size);
    dbg!(decoded_token.len());
    let total_size = (visible_size + opaque_size).into();
    let (message, signature) = decoded_token.split_at_mut(total_size);
    public_key.verify(message, signature)?;

    let (visible, opaque) = message.split_at_mut(visible_size.into());

    let decrypted = {
        let mut cipher = ChaCha20::new(&KEY.into(), &NONCE.into());
        cipher.apply_keystream(opaque);
        opaque
    };

    reader
        .build_token(visible, decrypted)
        .map_err(DecodeTokenError::Other)
}

pub fn to_str<T, W>(
    writer: &W,
    token: &T,
    private_key: &Ed25519KeyPair,
) -> Result<String, EncodeTokenError<W::Error>>
where
    W: TokenWriter<T>,
{
    const HEADER_SIZE: usize = 4;
    const MAX_SIGNATURE_SIZE: usize = 105;

    let (visible, opaque) = writer.bytes(token).map_err(EncodeTokenError::Other)?;

    let (visible, opaque) = (visible.as_ref(), opaque.as_ref());
    let (visible_len, opaque_len) = (visible.len(), opaque.len());

    let max_buf_size = (visible_len + opaque_len) + HEADER_SIZE + MAX_SIGNATURE_SIZE;
    let mut message = Vec::with_capacity(max_buf_size);

    let opaque = {
        message.extend_from_slice(visible);
        message.extend_from_slice(opaque);
        &mut message[visible.len()..]
    };

    let mut cipher = ChaCha20::new(&KEY.into(), &NONCE.into());
    cipher.apply_keystream(opaque);

    let sig = private_key.sign(&message);
    message.extend_from_slice(sig.as_ref());

    let previous_len = message.len();
    message.extend_from_slice(&[0; 4]);
    message.copy_within(..previous_len, 4);
    let a = u16::try_from(visible_len).unwrap().to_le_bytes();
    message[0] = a[0];
    message[1] = a[1];

    let b = u16::try_from(opaque_len).unwrap().to_le_bytes();
    message[2] = b[0];
    message[3] = b[1];

    Ok(BASE64_URL_SAFE.encode(message))
}

mod rocket;
