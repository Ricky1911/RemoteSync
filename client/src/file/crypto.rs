use std::path::Path;

use aes_gcm::{
    Aes256Gcm,
    aead::{
        OsRng,
        stream::{DecryptorBE32, EncryptorBE32},
    },
};
use rsa::rand_core::RngCore as _;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct AesKey {
    pub key: [u8; 32],
    pub nonce: [u8; 7],
}

pub fn generate_aes_keys() -> AesKey {
    let mut key: [u8; 32] = [0; 32];
    let mut nonce: [u8; 7] = [0; 7];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);
    AesKey { key, nonce }
}

const CHUNCK_SIZE: usize = 64 * 1024;

pub async fn aes_encrypt_file(
    src_path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
    key: &AesKey,
) -> anyhow::Result<()> {
    let src_path = src_path.as_ref();
    let out_path = out_path.as_ref();
    if out_path.exists() {
        return Err(anyhow::Error::msg(format!(
            "File {} already exists",
            out_path.display()
        )));
    }

    let mut encryptor = EncryptorBE32::<Aes256Gcm>::new(&key.key.into(), &key.nonce.into());
    let in_file = tokio::fs::File::open(src_path).await?;
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
            .map_err(|_| anyhow::Error::msg("Failed to encrypt"))?;
        out_file.write_all(&buffer).await?;
    }
    buffer.truncate(0);
    in_file.read_to_end(&mut buffer).await?;
    encryptor
        .encrypt_last_in_place(&[], &mut buffer)
        .map_err(|_| anyhow::Error::msg("Failed to encrypt"))?;
    out_file.write_all(&buffer).await?;
    out_file.flush().await?;
    Ok(())
}

pub async fn aes_decrypt_file(
    src_path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
    key: &AesKey,
) -> anyhow::Result<()> {
    let src_path = src_path.as_ref();
    let out_path = out_path.as_ref();
    if out_path.exists() {
        return Err(anyhow::Error::msg(format!(
            "File {} already exists",
            out_path.display()
        )));
    }

    let mut decryptor = DecryptorBE32::<Aes256Gcm>::new(&key.key.into(), &key.nonce.into());
    let in_file = tokio::fs::File::open(src_path).await?;
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
            .map_err(|_| anyhow::Error::msg("Failed to decrypt"))?;
        out_file.write_all(&buffer).await?;
    }
    buffer.truncate(0);
    in_file.read_to_end(&mut buffer).await?;
    decryptor
        .decrypt_last_in_place(&[], &mut buffer)
        .map_err(|_| anyhow::Error::msg("Failed to decrypt"))?;
    out_file.write_all(&buffer).await?;
    out_file.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{Read as _, Write as _};

    use super::*;

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
        let enc_path = tmp_dir.path().join("enc");
        let dec_path = tmp_dir.path().join("dec");
        aes_encrypt_file(&tmp_file_path, &enc_path, &aes_key)
            .await
            .unwrap();
        aes_decrypt_file(&enc_path, &dec_path, &aes_key)
            .await
            .unwrap();
        let mut dec_data = Vec::new();
        std::fs::File::open(dec_path)
            .unwrap()
            .read_to_end(&mut dec_data)
            .unwrap();
        assert_eq!(data, dec_data);
        tmp_dir.close().unwrap();
    }
}
