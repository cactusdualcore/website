#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

pub use ring;

pub trait TokenWriter<T: ?Sized> {
    type Error;
    type Bytes: AsRef<[u8]>;

    fn bytes(&self, token: &T) -> Result<(Self::Bytes, Self::Bytes), Self::Error>;
}

pub trait TokenReader {
    type Error;
    type Token;

    fn build_token(&self, visible: &[u8], opaque: &[u8]) -> Result<Self::Token, Self::Error>;
}

pub trait Token {
    type Reader: TokenReader;
    type Writer: TokenWriter<Self>;
}

mod lambda;
mod web;

pub use web::{to_str, Auth, AuthRequestError, KeyProvider};
