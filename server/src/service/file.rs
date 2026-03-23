use actix_multipart::Multipart;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::web::Data;
use actix_web::{Error, HttpResponse, Responder, get, post, web};
use futures_util::StreamExt as _;
use serde_json::json;
use uuid::Uuid;
use std::fs::File;
use std::io::{Read as _, Write as _};

use crate::models;
#[post("file")]
pub async fn post_file(mut payload: Multipart, config: Data<models::AppConfig>) -> Result<HttpResponse, Error> {
    println!("\n---upload_file---\n");

    while let Some(Ok(mut field)) = payload.next().await {
        let content_disposition = field.content_disposition().unwrap();
        let file_name = content_disposition.get_filename().unwrap();

        // 获取临时目录路径
        let mut file_path = config.save_path.clone();
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


#[get("file/{uuid}")]
async fn get_file(path: web::Path<Uuid>, config: Data<models::AppConfig>) -> impl Responder {
    let file_path = config.save_path.join(path.to_string());
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

#[post("entry")]
async fn create_entry() -> impl Responder {
    let uuid = Uuid::new_v4();
    HttpResponse::Ok().json(json!({"uuid": uuid}))
}