extern crate argon2rs;
use argon2rs::argon2d_simple as kdf;

extern crate chacha20_poly1305_aead;
use chacha20_poly1305_aead::{encrypt, decrypt};

use std::io::{Read, Write};

const SALT: &str = "vRNsYGE64KkdvXA8zCcP234793kxJ8fD";

fn main() {
    let password = "password";
    let key = kdf(password, SALT);

    let nonce = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let aad = [1, 2, 3, 4];

    let path = std::path::Path::new("/home/kpimenov/crypto");
    if path.exists() {
        println!("File found");
        let mut tag = [0u8; 16];
        let mut ciphertext = vec![];

        let mut file = std::fs::File::open(path).expect("Failed to open file");

        file.read_exact(&mut tag).expect("Failed to read AEAD header");
        file.read_to_end(&mut ciphertext).expect("Failed to read encrypted data");

        let mut plaintext = Vec::with_capacity(ciphertext.len());

        decrypt(&key, &nonce, &aad, &ciphertext, &tag, &mut plaintext).expect("Decription failed");

        println!("Decrypted content is\n\t{:?}", plaintext);
    } else {
        println!("File not found, creating!");

        let plaintext = b"hello, world";
        println!("Pre-encryption content is\n\t{:?}", plaintext);

        let mut ciphertext = Vec::with_capacity(plaintext.len());

        let tag = encrypt(&key, &nonce, &aad, plaintext, &mut ciphertext).expect("Encryption failed");

        let mut file = std::fs::File::create(path).expect("Failed to create a file");
        file.write_all(&tag).expect("Failed to write AEAD header");
        file.write_all(&ciphertext).expect("Failed to write ciphertext");
    }
}