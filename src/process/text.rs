use std::path::Path;
use std::{fs, io::Read};

use anyhow::{Ok, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::Signature;
use ed25519_dalek::Verifier;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use crate::{cli::TextSignFormat, utils::get_reader};
use crate::{process_genpass, TextCryptFormat};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};

pub trait TextSign {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextVerify {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool>;
}

pub trait KeyLoader {
    fn load(path: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized; // 这里使用Sized这个trait来约束返回的Self。  Sized是一个core::marker的（没有函数的），编译器理解判断这个Self必须是有固定长度的
}

pub trait KeyGenerator {
    fn generate() -> Result<Vec<Vec<u8>>>;
}

pub trait TextEncrypt {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

pub trait TextDecrypt {
    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>>;
}

struct Blake3 {
    key: [u8; 32],
}

struct Ed25519Signer {
    /// Sign the data from the reader and return the signature
    key: SigningKey,
}

struct Ed25519Verifier {
    /// Verify the data from the reader with the signature
    key: VerifyingKey,
}

struct Chacha20poly1305Obj {
    key: Key,
    nonce: Nonce,
}

pub fn process_text_sign(input: &str, key: &str, format: TextSignFormat) -> Result<String> {
    let mut reader = get_reader(input)?;
    let signed = match format {
        TextSignFormat::Blake3 => {
            let signer = Blake3::load(key)?;
            signer.sign(&mut reader)?
        }
        TextSignFormat::Ed25519 => {
            let signer = Ed25519Signer::load(key)?;
            signer.sign(&mut reader)?
        }
    };

    let signed = URL_SAFE_NO_PAD.encode(signed);

    Ok(signed)
}

pub fn process_text_verify(
    input: &str,
    key: &str,
    format: TextSignFormat,
    sig: &str,
) -> Result<bool> {
    let mut reader = get_reader(input)?;
    let sig = URL_SAFE_NO_PAD.decode(sig)?;
    match format {
        TextSignFormat::Blake3 => {
            let verifier = Blake3::load(key)?;
            verifier.verify(&mut reader, &sig)
        }
        TextSignFormat::Ed25519 => {
            let verifier = Ed25519Verifier::load(key)?;
            verifier.verify(&mut reader, &sig)
        }
    }
    // print!("{}",verified);
}

pub fn process_generate_key(format: TextSignFormat) -> Result<Vec<Vec<u8>>> {
    match format {
        TextSignFormat::Blake3 => Blake3::generate(),
        TextSignFormat::Ed25519 => Ed25519Signer::generate(),
    }
}

pub fn process_encrypt(
    input: &str,
    key: &str,
    nonce: &str,
    format: TextCryptFormat,
) -> Result<String> {
    let mut reader = get_reader(input)?;
    let encrypted = match format {
        TextCryptFormat::Chacha20poly1305 => {
            // Chacha20poly1305Obj::try_new(&key.as_bytes()[..32].try_into().unwrap(), &nonce.as_bytes()[..12].try_into().unwrap())
            // let key:[u8;32] = key.as_bytes().try_into().unwrap();
            // let nonce:[u8;12] = nonce.as_bytes().try_into().unwrap();
            let key = str_to_u8_array_32(key);
            let nonce = str_to_u8_array_12(nonce);
            // println!("key {:?}",key);
            // println!("nonce {:?}",nonce);
            let c = Chacha20poly1305Obj::try_new(&key, &nonce)?;
            c.encrypt(&mut reader)?
        }
    };
    let encrypted = URL_SAFE_NO_PAD.encode(encrypted);
    Ok(encrypted)
}

pub fn process_decrypt(
    input: &str,
    key: &str,
    nonce: &str,
    format: TextCryptFormat,
) -> Result<String> {
    let mut reader = get_reader(input)?;

    let data = match format {
        TextCryptFormat::Chacha20poly1305 => {
            let key = str_to_u8_array_32(key);
            let nonce = str_to_u8_array_12(nonce);
            let c = Chacha20poly1305Obj::try_new(&key, &nonce)?;
            c.decrypt(&mut reader)?
        }
    };

    Ok(String::from_utf8(data)?)
}

impl TextSign for Blake3 {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        // TODO: improve perf by reading in chunks
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec())
    }
}

impl TextVerify for Blake3 {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let hash = blake3::keyed_hash(&self.key, &buf).as_bytes().to_vec();
        Ok(hash == sig)
    }
}

impl TextSign for Ed25519Signer {
    fn sign(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let signature: Signature = self.key.sign(&buf);
        Ok(signature.to_bytes().to_vec())
    }
}

impl TextVerify for Ed25519Verifier {
    fn verify(&self, reader: &mut dyn Read, sig: &[u8]) -> Result<bool> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let sig = Signature::from_bytes(sig.try_into()?);
        Ok(self.key.verify(&buf, &sig).is_ok())
    }
}

impl KeyLoader for Blake3 {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519Signer {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyLoader for Ed25519Verifier {
    fn load(path: impl AsRef<Path>) -> Result<Self> {
        let key = fs::read(path)?;
        Self::try_new(&key)
    }
}

impl KeyGenerator for Blake3 {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let key = process_genpass(32, true, true, true, true)?;
        let key = key.as_bytes().to_vec();
        Ok(vec![key])
    }
}

impl KeyGenerator for Ed25519Signer {
    fn generate() -> Result<Vec<Vec<u8>>> {
        let mut csprng = OsRng;
        let sk = SigningKey::generate(&mut csprng);
        let pk = sk.verifying_key().to_bytes().to_vec();
        let sk = sk.as_bytes().to_vec();
        Ok(vec![sk, pk])
    }
}

impl TextEncrypt for Chacha20poly1305Obj {
    fn encrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let cipher = ChaCha20Poly1305::new(&self.key);
        let ciphertext: Vec<u8> = cipher
            .encrypt(&self.nonce, buf.as_ref())
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(ciphertext)
    }
}

impl TextDecrypt for Chacha20poly1305Obj {
    fn decrypt(&self, reader: &mut dyn Read) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        let data = URL_SAFE_NO_PAD.decode(buf)?;

        let cipher = ChaCha20Poly1305::new(&self.key);
        let data: Vec<u8> = cipher
            .decrypt(&self.nonce, &*data)
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(data)
    }
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = &key[..32];
        let key: [u8; 32] = key.try_into().unwrap();
        let signer = Blake3::new(key);
        Ok(signer)
    }
}

impl Ed25519Signer {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }
    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = SigningKey::from_bytes(key.try_into()?);
        Ok(Self::new(key))
    }
}

impl Ed25519Verifier {
    pub fn new(key: VerifyingKey) -> Self {
        Self { key }
    }
    pub fn try_new(key: &[u8]) -> Result<Self> {
        let key = VerifyingKey::from_bytes(key.try_into()?)?;
        Ok(Self::new(key))
    }
}

impl Chacha20poly1305Obj {
    pub fn new(key: Key, nonce: Nonce) -> Self {
        Self { key, nonce }
    }
    pub fn try_new(key: &[u8; 32], nonce: &[u8; 12]) -> Result<Self> {
        let key = Key::from_slice(key);
        let nonce = Nonce::from_slice(nonce);
        Ok(Self::new(*key, *nonce))
    }
}

fn str_to_u8_array_32(input: &str) -> [u8; 32] {
    let mut byte_array = [0u8; 32];
    let bytes = input.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if i < 32 {
            byte_array[i] = item;
        } else {
            break;
        }
    }

    byte_array
}

fn str_to_u8_array_12(input: &str) -> [u8; 12] {
    let mut byte_array = [0u8; 12];
    let bytes = input.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if i < 12 {
            byte_array[i] = item;
        } else {
            break;
        }
    }

    byte_array
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake_sign_verify() -> Result<()> {
        let data = b"Hello World";
        let blake3 = Blake3::load("./fixtures/blake3.txt")?;
        let sig = blake3.sign(&mut &data[..])?;
        assert!(blake3.verify(&mut &data[..], &sig)?);
        Ok(())
    }

    #[test]
    fn test_ed25519_sign_verify() -> Result<()> {
        let data = b"Hello World";

        let sk = Ed25519Signer::load("./fixtures/ed25519.sk")?;
        let pk = Ed25519Verifier::load("./fixtures/ed25519.pk")?;

        let sig = sk.sign(&mut &data[..])?;

        assert!(pk.verify(&mut &data[..], &sig)?);
        Ok(())
    }
}
