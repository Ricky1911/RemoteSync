use std::path::PathBuf;

use clap::{Parser, Subcommand};
use uuid::Uuid;

use crate::{
    file::{
        archive::{pack_archive, unpack_archive},
        crypto::generate_aes_keys,
    },
    network::Client,
};
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    Sync,
    Push { entry: Uuid, src: PathBuf },
    Pull { entry: Uuid, dest: PathBuf },
    CreateUser,
    CreateEntry,
}

pub async fn push(entry: Uuid, src: PathBuf, client: &mut Client) -> anyhow::Result<()> {
    let aes_key = generate_aes_keys();
    let temp_dir = tempfile::tempdir()?;
    let dest_path = temp_dir.path().join(Uuid::new_v4().to_string());
    pack_archive(src, &dest_path, &aes_key).await?;
    client.upload(entry, dest_path, &aes_key).await?;
    Ok(())
}

pub async fn pull(entry: Uuid, dest: PathBuf, client: &mut Client) -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let save_dir = temp_dir.path().join(Uuid::new_v4().to_string());
    let (download_path, aes_key) = client.download(entry, save_dir).await?;
    unpack_archive(download_path, dest, &aes_key).await
}
