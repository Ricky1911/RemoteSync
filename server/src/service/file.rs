use actix_multipart::Multipart;
use actix_web::{Error, HttpResponse, post};
use futures_util::StreamExt as _;
use std::env;
use std::fs::File;
use std::io::Write as _;
#[post("upload_file")]
pub async fn post_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    println!("\n---upload_file---\n");

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition().unwrap();
        let file_name = content_disposition.get_filename().unwrap();

        // 获取临时目录路径
        let mut file_path = env::temp_dir();
        file_path.push(file_name);
        println!(
            "---full file_path:{}, file_name:{}",
            file_path.display(),
            file_name
        );

        let mut file = File::create(file_path)?;
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file.write_all(&data)?;
        }
    }
    Ok(HttpResponse::Ok().finish())
}
