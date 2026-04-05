use common::models::NewUpdate;
use reqwest::{
    StatusCode,
    header::{self, HeaderMap, HeaderValue},
    multipart::Part,
};
use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs8::{
        DecodePrivateKey, DecodePublicKey, EncodePrivateKey as _, EncodePublicKey, LineEnding,
    },
};
use url::Url;
use uuid::Uuid;

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::config::ClientConfig;

pub struct Client {
    client: reqwest::Client,
    api_url: Url,
    public_key: RsaPublicKey,
    private_key: RsaPrivateKey,
}

impl Client {
    pub async fn init(config: ClientConfig) -> Self {
        let public_key = RsaPublicKey::read_public_key_pem_file(config.public_pem)
            .expect("Failed to load public key");
        let private_key = RsaPrivateKey::read_pkcs8_pem_file(config.private_pem)
            .expect("Failed to load private key");
        let client = Client::login(config.username, config.password, &config.api_url)
            .await
            .expect("Failed to login");
        Client {
            client,
            api_url: config.api_url,
            public_key,
            private_key,
        }
    }

    pub async fn upload<T>(&mut self, entry: Uuid, path: T) -> anyhow::Result<()>
    where
        T: AsRef<Path>,
    {
        let signature = common::crypto::sign_file(&self.private_key, &path).await?;
        let update_info = NewUpdate {
            aes_key: Vec::new(),
            signature,
        };
        let metadata_part = Part::bytes(postcard::to_allocvec(&update_info)?);
        let file_part = Part::file(path.as_ref()).await?;
        let form = reqwest::multipart::Form::new()
            .part("metadata", metadata_part)
            .part("file", file_part);
        let url = self
            .api_url
            .join(&format!("file/{}", &entry.to_string()))
            .unwrap();
        let response = self.client.post(url).multipart(form).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::Error::msg("Unexpected response"))
        }
    }

    pub async fn download<T>(&mut self, entry: Uuid, save_dir: T) -> anyhow::Result<PathBuf>
    where
        T: AsRef<Path>,
    {
        if !save_dir.as_ref().is_dir() {
            return Err(anyhow::Error::msg(format!(
                "{:?} is not a directory",
                save_dir.as_ref()
            )));
        }
        let url = self
            .api_url
            .join(&format!("file/{}", &entry.to_string()))
            .unwrap();
        let response = self.client.get(url).send().await?;
        let fname = entry.to_string();
        let path = save_dir.as_ref().join(&fname);
        let mut dest = std::fs::File::create(&path)?;
        dest.write(&response.bytes().await.unwrap())?;
        Ok(path)
    }

    pub async fn create_entry(&mut self) -> anyhow::Result<Uuid> {
        let response = self
            .client
            .post(self.api_url.join("entry").unwrap())
            .send()
            .await?;
        if response.status().is_success() {
            let entry_info: common::models::EntryInfo = response.json().await?;
            Ok(entry_info.uuid)
        } else {
            panic!()
        }
    }

    async fn login(
        username: String,
        password: String,
        api_url: &Url,
    ) -> anyhow::Result<reqwest::Client> {
        let response = reqwest::Client::new()
            .post(api_url.join("login")?)
            .json(&common::models::LoginRequest {
                name: username,
                password: password,
            })
            .send()
            .await?;
        let token = response.json::<common::models::TokenResponse>().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token.token))?,
        );
        Ok(reqwest::Client::builder()
            .default_headers(headers)
            .build()?)
    }
}

pub async fn create_user(
    username: String,
    password: String,
    api_url: Url,
    public_pem: PathBuf,
    private_pem: PathBuf,
) -> anyhow::Result<()> {
    let (private_key, public_key) =
        common::crypto::generate_keys().expect("Failed to generate RSA Keys");
    let client = reqwest::Client::new();

    let response = client
        .post(api_url.join("user").unwrap())
        .json(&common::models::NewUser {
            name: username,
            password: password,
            public_key: public_key.clone(),
        })
        .send()
        .await?;
    match response.status() {
        StatusCode::OK => {
            private_key
                .write_pkcs8_pem_file(private_pem, LineEnding::CRLF)
                .expect("Failed to write public pem file");
            public_key
                .write_public_key_pem_file(public_pem, LineEnding::CRLF)
                .expect("Failed to write private pem file");
            Ok(())
        }
        StatusCode::CONFLICT => {
            panic!("Username conflict")
        }
        _ => {
            panic!(
                "Unexpected error with response: {}",
                response.text().await.unwrap()
            )
        }
    }
}
