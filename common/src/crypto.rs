use std::path::Path;

use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::rand_core::OsRng;
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier as _};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::io::AsyncReadExt;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error occurred")]
    IoError(#[from] std::io::Error),
    #[error("Failed to generate key")]
    KeyGenerateError,
    #[error("Failed to encrypt key")]
    EncryptError,
    #[error("Failed to decrypt key")]
    DecryptError,
    #[error("Failed to serialzie")]
    SerializeError,
    #[error("Failed to deserialize")]
    DeserializeError,
}

pub fn generate_keys() -> Result<(RsaPrivateKey, RsaPublicKey), Error> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|_| Error::KeyGenerateError)?;
    let public_key = RsaPublicKey::from(&private_key);
    Ok((private_key, public_key))
}

pub fn encrypt_data(public_key: &RsaPublicKey, data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut rng = OsRng;
    public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data)
        .map_err(|_| Error::EncryptError)
}

pub fn decrypt_data(private_key: &RsaPrivateKey, encrypted_data: &[u8]) -> Result<Vec<u8>, Error> {
    private_key
        .decrypt(Pkcs1v15Encrypt, encrypted_data)
        .map_err(|_| Error::DecryptError)
}

pub fn sign_data(private_key: &RsaPrivateKey, data: &[u8]) -> Vec<u8> {
    let signing_key = SigningKey::<Sha256>::new_unprefixed(private_key.clone());
    let mut rng = OsRng;
    signing_key.sign_with_rng(&mut rng, data).to_vec()
}

pub fn verify_signature(
    public_key: &RsaPublicKey,
    data: &[u8],
    signature: &[u8],
) -> Result<bool, Error> {
    let signature = Signature::try_from(signature).map_err(|_| Error::DeserializeError)?;
    let verifying_key = VerifyingKey::<Sha256>::new_unprefixed(public_key.clone());
    Ok(verifying_key.verify(data, &signature).is_ok())
}

pub fn public_key_to_bytes(public_key: &RsaPublicKey) -> Result<Vec<u8>, Error> {
    postcard::to_allocvec(public_key).map_err(|_| Error::SerializeError)
}

pub fn bytes_to_public_key(bytes: &[u8]) -> Result<RsaPublicKey, Error> {
    postcard::from_bytes(bytes).map_err(|_| Error::DeserializeError)
}

pub async fn stream_hash<T>(path: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<Path>,
{
    let mut hasher = sha2::Sha256::new();
    let mut file = tokio::fs::File::open(path.as_ref()).await?;
    let mut buffer = [0; 8192];
    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_vec())
}

pub async fn sign_file<T>(private_key: &RsaPrivateKey, path: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<Path>,
{
    let hash = stream_hash(path).await?;
    Ok(sign_data(private_key, &hash))
}

pub async fn verify_file<T>(
    public_key: &RsaPublicKey,
    path: T,
    signature: &[u8],
) -> Result<bool, Error>
where
    T: AsRef<Path>,
{
    let hash = stream_hash(path).await?;
    verify_signature(public_key, &hash, signature)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_encrypt_and_decrypt() {
        let (private_key, public_key) = generate_keys().unwrap();
        let data = b"Hello, RSA in Rust!";
        let encrypted_data = encrypt_data(&public_key, data).unwrap();
        dbg!("Encrypted Data: {:?}", &encrypted_data);
        let decrypted_data = decrypt_data(&private_key, &encrypted_data);
        assert!(decrypted_data.unwrap() == data)
    }

    #[test]
    fn test_sign_and_verify() {
        let (private_key, public_key) = generate_keys().unwrap();
        let data = b"Hello, RSA in Rust!";
        let signature = sign_data(&private_key, data);
        dbg!("Signature: {:?}", &signature);
        let is_valid = verify_signature(&public_key, data, &signature);
        assert!(is_valid.unwrap())
    }
}
