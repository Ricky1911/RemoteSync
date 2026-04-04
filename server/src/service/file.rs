use actix_multipart::Multipart;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse, Responder, get, post, web};
use futures_util::StreamExt as _;
use std::fs::File;
use std::io::{Read as _, Write as _};
use uuid::Uuid;

use crate::config;
#[post("file/{entry}")]
pub async fn upload_file(
    entry: web::Path<Uuid>,
    mut payload: Multipart,
    config: Data<config::ServerConfig>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    match req.extensions().get::<Uuid>() {
        Some(user_id) => {
            println!("\n---upload_file---\n");

            while let Some(Ok(mut field)) = payload.next().await {
                let content_disposition = field.content_disposition().unwrap();
                let file_name = content_disposition.get_filename().unwrap();
                let file_dir = config.save_path.join(user_id.to_string());
                if !file_dir.exists() {
                    std::fs::create_dir(&file_dir)?;
                }
                let file_path = file_dir.join(entry.to_string());
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
        None => Ok(HttpResponse::Unauthorized().body("Invalid authorization token")),
    }
}

#[get("file/{entry}")]
async fn download_file(
    entry: web::Path<Uuid>,
    config: Data<config::ServerConfig>,
    req: HttpRequest,
) -> impl Responder {
    match req.extensions().get::<Uuid>() {
        Some(user_id) => {
            let file_path = config
                .save_path
                .join(user_id.to_string())
                .join(entry.to_string());
            let mut file = File::open(&file_path).expect("Can't open file!");

            // 读取文件内容
            let mut chunk = vec![];
            if let Err(e) = file.read_to_end(&mut chunk) {
                // println!("Err => {}", e);
                return HttpResponse::Ok().body(e.to_string());
            }

            // 得到文件名
            let file_name = file_path.file_name().unwrap().to_str().unwrap();

            let cd = ContentDisposition {
                disposition: DispositionType::FormData,
                parameters: vec![
                    DispositionParam::Name(String::from("upload")),
                    DispositionParam::Filename(file_name.to_string()),
                ],
            };

            let mut builder = HttpResponse::Ok();
            builder.insert_header((actix_web::http::header::CONTENT_DISPOSITION, cd));

            builder.body(chunk)
        }
        None => HttpResponse::Unauthorized().body("Invalid authorization token"),
    }
}

#[post("entry")]
async fn create_entry() -> impl Responder {
    let uuid = Uuid::new_v4();
    HttpResponse::Ok().json(common::models::EntryInfo { uuid })
}
