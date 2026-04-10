use std::path::{Path, PathBuf};

use aes_gcm::Aes256Gcm;
use aes_gcm::aead::stream::{DecryptorBE32, EncryptorBE32};
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::rand_core::{OsRng, RngCore};
use rsa::signature::{RandomizedSigner as _, SignatureEncoding as _, Verifier as _};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use thiserror::Error;
use tokio::io::AsyncReadExt as _;
use tokio::io::AsyncWriteExt as _;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error occurred")]
    IoError(#[from] std::io::Error),
    #[error("Failed to generate key")]
    KeyGenerateError,
    #[error("Failed to encrypt")]
    EncryptError,
    #[error("Failed to decrypt")]
    DecryptError,
    #[error("Failed to serialzie")]
    SerializeError,
    #[error("Failed to deserialize")]
    DeserializeError,
}

pub fn generate_rsa_keys() -> Result<(RsaPrivateKey, RsaPublicKey), Error> {
    let mut rng = OsRng;
    let private_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|_| Error::KeyGenerateError)?;
    let public_key = RsaPublicKey::from(&private_key);
    Ok((private_key, public_key))
}

pub fn rsa_encrypt_data(public_key: &RsaPublicKey, data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut rng = OsRng;
    public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, data)
        .map_err(|_| Error::EncryptError)
}

pub fn rsa_decrypt_data(
    private_key: &RsaPrivateKey,
    encrypted_data: &[u8],
) -> Result<Vec<u8>, Error> {
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

#[derive(Serialize, Deserialize)]
pub struct AesKey {
    pub key: [u8; 32],
    pub nonce: [u8; 7],
}

pub fn generate_aes_keys() -> AesKey {
    let mut key: [u8; 32] = [0; 32];
    let mut nonce: [u8; 7] = [0; 7];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);
    AesKey {
        key: key,
        nonce: nonce,
    }
}

const CHUNCK_SIZE: usize = 64 * 1024;

pub async fn aes_encrypt_file<T>(path: T, key: &AesKey) -> Result<PathBuf, Error>
where
    T: AsRef<Path>,
{
    let mut encryptor = EncryptorBE32::<Aes256Gcm>::new(&key.key.into(), &key.nonce.into());
    let out_path = path.as_ref().with_added_extension("enc");
    let in_file = tokio::fs::File::open(path).await?;
    let file_length = in_file.metadata().await?.len();
    let chunck_count = if file_length % CHUNCK_SIZE as u64 == 0 {
        file_length / CHUNCK_SIZE as u64
    } else {
        file_length / CHUNCK_SIZE as u64 + 1
    };
    let mut in_file = tokio::io::BufReader::new(in_file);
    let mut out_file = tokio::io::BufWriter::new(tokio::fs::File::create(&out_path).await?);

    let mut buffer = Vec::with_capacity(CHUNCK_SIZE + 16);
    for _ in 0..chunck_count - 1 {
        buffer.resize(CHUNCK_SIZE, 0);
        in_file.read_exact(&mut buffer).await?;
        encryptor
            .encrypt_next_in_place(&[], &mut buffer)
            .map_err(|_| Error::EncryptError)?;
        out_file.write_all(&buffer).await?;
    }
    buffer.truncate(0);
    in_file.read_to_end(&mut buffer).await?;
    encryptor
        .encrypt_last_in_place(&[], &mut buffer)
        .map_err(|_| Error::EncryptError)?;
    out_file.write_all(&buffer).await?;
    out_file.flush().await?;
    Ok(out_path)
}

pub async fn aes_decrypt_file<T>(path: T, key: &AesKey) -> Result<PathBuf, Error>
where
    T: AsRef<Path>,
{
    let mut decryptor = DecryptorBE32::<Aes256Gcm>::new(&key.key.into(), &key.nonce.into());
    let out_path = path.as_ref().with_added_extension("dec");
    let in_file = tokio::fs::File::open(path).await?;
    let file_length = in_file.metadata().await?.len();
    let chunck_count = if file_length % (CHUNCK_SIZE + 16) as u64 == 0 {
        file_length / (CHUNCK_SIZE + 16) as u64
    } else {
        file_length / (CHUNCK_SIZE + 16) as u64 + 1
    };
    let mut in_file = tokio::io::BufReader::new(in_file);
    let mut out_file = tokio::io::BufWriter::new(tokio::fs::File::create(&out_path).await?);

    let mut buffer = Vec::with_capacity(CHUNCK_SIZE + 16);
    for _ in 0..chunck_count - 1 {
        buffer.resize(CHUNCK_SIZE + 16, 0);
        in_file.read_exact(&mut buffer).await?;
        decryptor
            .decrypt_next_in_place(&[], &mut buffer)
            .map_err(|_| Error::DecryptError)?;
        out_file.write_all(&buffer).await?;
    }
    buffer.truncate(0);
    in_file.read_to_end(&mut buffer).await?;
    decryptor
        .decrypt_last_in_place(&[], &mut buffer)
        .map_err(|_| Error::DecryptError)?;
    out_file.write_all(&buffer).await?;
    out_file.flush().await?;
    Ok(out_path)
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};

    use super::*;
    #[test]
    fn test_rsa_encrypt_and_decrypt() {
        let (private_key, public_key) = generate_rsa_keys().unwrap();
        let data = b"Hello, RSA in Rust!";
        let encrypted_data = rsa_encrypt_data(&public_key, data).unwrap();
        let decrypted_data = rsa_decrypt_data(&private_key, &encrypted_data);
        assert!(decrypted_data.unwrap() == data)
    }

    #[test]
    fn test_sign_and_verify() {
        let (private_key, public_key) = generate_rsa_keys().unwrap();
        let data = b"Hello, RSA in Rust!";
        let signature = sign_data(&private_key, data);
        let is_valid = verify_signature(&public_key, data, &signature);
        assert!(is_valid.unwrap())
    }

    #[tokio::test]
    async fn test_aes_encrypt_and_decrypt() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_file_path = tmp_dir.path().join("test");
        let mut tmp_file = std::fs::File::create(&tmp_file_path).unwrap();
        let data: Vec<u8> = std::iter::repeat_with(|| 0u8)
            .take(CHUNCK_SIZE * 2)
            .collect();
        tmp_file.write_all(&data).unwrap();
        let aes_key = generate_aes_keys();
        let enc_path = aes_encrypt_file(&tmp_file_path, &aes_key).await.unwrap();
        let dec_path = aes_decrypt_file(enc_path, &aes_key).await.unwrap();
        let mut dec_data = Vec::new();
        std::fs::File::open(dec_path)
            .unwrap()
            .read_to_end(&mut dec_data)
            .unwrap();
        assert_eq!(data, dec_data);
        tmp_dir.close().unwrap();
    }
}
