use reqwest::multipart::Part;

use std::path::Path;

pub async fn upload<T>(path: T)
where
    T: AsRef<Path>,
{
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new().part("file", Part::file(path).await.unwrap());
    let response = client
        .post("http://localhost:8080/upload_file")
        .multipart(form)
        .send()
        .await
        .unwrap();
    println!("{:?}", response);
}
