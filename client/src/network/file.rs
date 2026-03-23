use reqwest::multipart::Part;
use uuid::Uuid;

use std::{error::Error, io::Write, path::Path};

pub async fn upload<T>(path: T)
where
    T: AsRef<Path>,
{
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new().part("file", Part::file(path).await.unwrap());
    let response = client
        .post("http://localhost:8080/file")
        .multipart(form)
        .send()
        .await
        .unwrap();
    println!("{:?}", response);
}

pub async fn download(uuid: Uuid) -> Result<(), Box<dyn Error>> {
    let mut tmp_dir = std::env::temp_dir();
    let target = format!("http://localhost:8080/file/{}", uuid);
    let response = reqwest::get(target).await?;

    let mut dest = {
        let fname = uuid.to_string();

        println!("file to download: '{}'", fname);
        tmp_dir.push(fname);
        println!("will be located under: '{:?}'", tmp_dir);
        std::fs::File::create(tmp_dir)?
    };
    dest.write(&response.bytes().await.unwrap())?;
    Ok(())
}
