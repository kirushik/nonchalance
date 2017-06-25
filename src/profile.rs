use ring_pwhash::scrypt::{scrypt, ScryptParams};
use base64;

#[derive(Debug)]
pub struct Profile {
    identity: String,
    root: [u8; 32],
}

impl Profile {
    pub fn new(identity: &str, password: &str) -> Profile {
        let mut root = [0u8; 32];
        scrypt(password.as_bytes(),
               identity.as_bytes(),
               &ScryptParams::new(18, 8, 6),
               &mut root);
        Profile {
            identity: identity.to_string(),
            root: root,
        }
    }

    fn root_base64(&self) -> String {
        base64::encode(&self.root)
    }

    pub fn sltoken_for(&self, provider: &str) -> String {
        format!("{},{}", self.root_base64(), provider)
    }
}