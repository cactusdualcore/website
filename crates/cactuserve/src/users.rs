use bitflags::bitflags;
use cactuserve_sessions::{
    ring::signature::{KeyPair, UnparsedPublicKey, ED25519},
    KeyProvider,
};
use rocket::http::hyper::body::Buf;
use smallvec::SmallVec;

use cactuserve_sessions::ring::{rand, signature};
use std::{
    fs::OpenOptions,
    io::{Read as _, Write as _},
    sync::OnceLock,
};

pub use cactuserve_sessions::ring::signature::Ed25519KeyPair;

pub fn key_pair() -> &'static Ed25519KeyPair {
    static KEY_PAIR: OnceLock<Ed25519KeyPair> = OnceLock::new();

    KEY_PAIR.get_or_init(|| {
        const KEY_FILE: &str = "session_key.pem";

        let pkcs8_bytes = match OpenOptions::new().read(true).open(KEY_FILE) {
            Ok(mut file) => {
                eprintln!("Found session key.");
                let mut buf = Vec::new();
                let _ = file.read_to_end(&mut buf).unwrap();

                let pem_encoded = pem::parse(&buf).unwrap();
                pem_encoded.contents().to_vec()
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                eprintln!("Generating session key.");
                let rng = rand::SystemRandom::new();
                let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();

                let text = {
                    let data = pem::Pem::new("PRIVATE KEY", pkcs8_bytes.as_ref());
                    pem::encode(&data)
                };

                let mut file = OpenOptions::new()
                    .create_new(true)
                    .truncate(true)
                    .write(true)
                    .open(KEY_FILE)
                    .unwrap();

                file.write_all(text.as_bytes()).unwrap();
                pkcs8_bytes.as_ref().to_vec()
            }
            Err(_err) => todo!(),
        };

        signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap()
    })
}

pub struct User {
    pub username: String,
    pub scope: Scope,
}

impl User {
    pub fn username(&self) -> &str {
        &self.username
    }

    pub const fn scope(&self) -> Scope {
        self.scope
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Scope: u16 {
        const Awesome = 1 << 0;
    }
}

#[derive(Default)]
pub struct Reader;

impl cactuserve_sessions::TokenReader for Reader {
    type Error = anyhow::Error;

    type Token = User;

    fn build_token(&self, visible: &[u8], mut opaque: &[u8]) -> Result<Self::Token, Self::Error> {
        let username = String::from_utf8_lossy(visible).into_owned();
        let scope = Scope::from_bits_truncate(opaque.get_u16_le());
        assert_eq!(opaque.len(), 0);
        Ok(User { username, scope })
    }
}

#[derive(Default)]
pub struct Writer;

impl cactuserve_sessions::TokenWriter<User> for Writer {
    type Bytes = SmallVec<[u8; 2]>;
    type Error = anyhow::Error;

    fn bytes(&self, token: &User) -> Result<(Self::Bytes, Self::Bytes), Self::Error> {
        let username = SmallVec::from_slice(token.username.as_bytes());
        let scope = token.scope.bits().to_le_bytes().into();
        Ok((username, scope))
    }
}

pub struct Provider;

impl KeyProvider for Provider {
    type Bytes = Vec<u8>;

    fn public_key() -> UnparsedPublicKey<Self::Bytes> {
        let key_pair = key_pair();
        let public_key_bytes = key_pair.public_key().as_ref();
        UnparsedPublicKey::new(&ED25519, public_key_bytes.to_vec())
    }
}
