#[macro_use]
extern crate serde_derive;
extern crate bincode;
use bincode::{serialize, deserialize, Infinite};

extern crate ring;
use ring::aead::{SealingKey, OpeningKey, seal_in_place, open_in_place};
use ring::aead::CHACHA20_POLY1305 as CYPHER;

extern crate rand;
use rand::random;

extern crate ring_pwhash;
use ring_pwhash::scrypt::{scrypt, ScryptParams};

use std::io::{Read, Write};

const SALT: &str = "vRNsYGE64KkdvXA8zCcP234793kxJ8fD";

#[derive(Serialize, Deserialize, Debug)]
struct Foo {
    bar: String,
    baz: Option<Vec<u64>>
}

fn main() {
    let password = "password";
    let mut key = [0u8; 32];
    scrypt(password.as_bytes(), SALT.as_bytes(), &ScryptParams::new(17, 8, 2), &mut key);

    let path = std::path::Path::new("/home/kpimenov/crypto");
    if path.exists() {
        println!("File found");
        let mut file = std::fs::File::open(path).expect("Failed to open file");

        let mut nonce = [0u8; 12];
        file.read_exact(&mut nonce).expect("Failed to read nonce header");
        
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).expect("Failed to read encrypted data");

        let key = OpeningKey::new(&CYPHER, &key).expect("Failed to initialize an OpeningKey");
        let plaintext = open_in_place(&key, &nonce, &[], 0, &mut buffer).expect("Decryption failed");

        let data = deserialize::<Foo>(&plaintext).expect("Deserialization failed!");

        println!("Decrypted content is\n\t{:?}", data);
    } else {
        println!("File not found, creating!");

        let plaintext = Foo{
            bar: "hey-hey!".into(),
            baz: None
        };

        println!("Pre-encryption content is\n\t{:?}", plaintext);

        let mut buffer = serialize(&plaintext, Infinite).expect("serialization failed");
        buffer.extend_from_slice(&[0u8; 32]);

        let mut file = std::fs::File::create(path).expect("Failed to create a file");

        let nonce: [u8; 12] = random();
        file.write_all(&nonce).expect("Failed to write nonce header");

        let key = SealingKey::new(&CYPHER, &key).expect("Failed to initialize a SealingKey");
        seal_in_place(&key, &nonce, &[], &mut buffer, CYPHER.tag_len()).expect("Encryption failed");

        file.write_all(&buffer).expect("Failed to write ciphertext");
    }
}