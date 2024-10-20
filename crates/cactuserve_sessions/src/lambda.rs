use std::convert::Infallible;

use super::{TokenReader, TokenWriter};

impl<B, F, T> TokenWriter<T> for F
where
    B: AsRef<[u8]>,
    F: Fn(&T) -> (B, B) + Clone,
{
    type Error = Infallible;
    type Bytes = B;

    #[inline]
    fn bytes(&self, token: &T) -> Result<(Self::Bytes, Self::Bytes), Self::Error> {
        Ok((self)(token))
    }
}

impl<F, T> TokenReader for F
where
    F: Fn(&[u8], &[u8]) -> T + Clone,
{
    type Error = Infallible;
    type Token = T;

    #[inline]
    fn build_token(&self, visible: &[u8], opaque: &[u8]) -> Result<Self::Token, Self::Error> {
        Ok((self)(visible, opaque))
    }
}
