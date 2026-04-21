use std::path::Path;

use uuid::Uuid;

use crate::file::{
    compress::{compress_archive, decompress_archive},
    crypto::{AesKey, aes_decrypt_file, aes_encrypt_file},
};

pub async fn pack_archive(
    src_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
    key: &AesKey,
) -> anyhow::Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_file = tmp_dir.path().join(Uuid::new_v4().to_string());
    compress_archive(src_path.as_ref(), &tmp_file)?;
    aes_encrypt_file(&tmp_file, dest_path.as_ref(), key).await?;
    Ok(())
}

pub async fn unpack_archive(
    src_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
    key: &AesKey,
) -> anyhow::Result<()> {
    let tmp_dir = tempfile::tempdir()?;
    let tmp_file = tmp_dir.path().join(Uuid::new_v4().to_string());
    aes_decrypt_file(src_path.as_ref(), &tmp_file, key).await?;
    decompress_archive(&tmp_file, dest_path.as_ref())?;
    Ok(())
}
