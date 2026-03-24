use reqwest::multipart::Part;
use uuid::Uuid;

use std::{error::Error, io::Write, path::Path};

use crate::config::ClientConfig;

pub struct Client {
    config: ClientConfig,
    client: reqwest::Client,
}

impl Client {
    pub fn new(config: ClientConfig) -> Self {
        Client {
            config,
            client: reqwest::Client::new(),
        }
    }

    pub async fn upload<T>(&mut self, uuid: Uuid, path: T)
    where
        T: AsRef<Path>,
    {
        let form = if let Ok(part) = Part::file(path.as_ref()).await {
            reqwest::multipart::Form::new().part("file", part)
        } else {
            dbg!(path.as_ref());
            panic!()
        };
        let response = self
            .client
            .post(
                self.config
                    .api_url
                    .join("file")
                    .unwrap()
                    .join(&uuid.to_string())
                    .unwrap(),
            )
            .multipart(form)
            .send()
            .await
            .unwrap();
        println!("{:?}", response);
    }

    pub async fn download<T>(&mut self, uuid: Uuid, save_dir: T) -> Result<(), Box<dyn Error>>
    where
        T: AsRef<Path>,
    {
        if !save_dir.as_ref().is_dir() {
            panic!("not a directory")
        }
        let response = self
            .client
            .get(
                self.config
                    .api_url
                    .join("file")
                    .unwrap()
                    .join(&uuid.to_string())
                    .unwrap(),
            )
            .send()
            .await?;

        let mut dest = {
            let fname = uuid.to_string();
            let path = save_dir.as_ref().join(&fname);
            println!("file to download: '{}'", fname);
            println!("will be located under: '{:?}'", path);
            std::fs::File::create(path)?
        };
        dest.write(&response.bytes().await.unwrap())?;
        Ok(())
    }

    pub async fn create_entry(&mut self) -> Result<Uuid, Box<dyn Error>> {
        let response = self
            .client
            .post(self.config.api_url.join("entry").unwrap())
            .send()
            .await?;
        if response.status().is_success() {
            let entry_info: common::models::EntryInfo = response.json().await?;
            Ok(entry_info.uuid)
        } else {
            panic!()
        }
    }
}
