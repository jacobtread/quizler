use rand::TryRng;
use ring::hmac::{self, HMAC_SHA256, Key, Tag};

pub struct SigningKey(Key);

impl AsRef<Key> for SigningKey {
    fn as_ref(&self) -> &Key {
        &self.0
    }
}

impl SigningKey {
    const KEY_LENGTH: usize = 64;

    #[inline]
    fn new(secret: &[u8; Self::KEY_LENGTH]) -> Self {
        Self(Key::new(HMAC_SHA256, secret))
    }

    #[inline]
    pub fn sign(&self, data: &[u8]) -> Tag {
        hmac::sign(&self.0, data)
    }

    #[inline]
    pub fn verify(&self, data: &[u8], tag: &[u8]) -> bool {
        hmac::verify(&self.0, data, tag).is_ok()
    }

    /// Generates a new signing key
    pub fn generate() -> Self {
        let mut secret = [0; Self::KEY_LENGTH];
        rand::rngs::SysRng
            .try_fill_bytes(&mut secret)
            .expect("failed to fill secret bytes");
        Self::new(&secret)
    }
}
