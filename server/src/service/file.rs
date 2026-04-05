use actix_multipart::Multipart;
use actix_web::http::header::{ContentDisposition, DispositionParam, DispositionType};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage, HttpRequest, HttpResponse, Responder, get, post, web};
use common::models::NewUpdate;
use diesel::{
    BoolExpressionMethods as _, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl,
    insert_into,
};
use futures_util::StreamExt as _;
use sha2::Digest;
use std::fs::File;
use std::io::{Read as _, Write as _};
use uuid::Uuid;

use crate::service::user::User;
use crate::{DbPool, config};

#[derive(Insertable, Queryable)]
#[diesel(table_name = crate::schema::entries)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct Entry {
    id: Uuid,
    user_id: Uuid,
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = crate::schema::updates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct Update {
    id: Uuid,
    entry_id: Uuid,
    created: chrono::NaiveDateTime,
    aes_key: Vec<u8>,
    sig: Vec<u8>,
}

impl Update {
    fn new(id: Uuid, entry_id: Uuid, aes_key: Vec<u8>, sig: Vec<u8>) -> Self {
        Update {
            id,
            entry_id,
            created: chrono::Local::now().naive_local(),
            aes_key,
            sig,
        }
    }
}

#[post("file/{entry}")]
async fn upload_file(
    entry: web::Path<Uuid>,
    mut payload: Multipart,
    config: Data<config::ServerConfig>,
    db_pool: Data<DbPool>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    match req.extensions().get::<Uuid>() {
        Some(user_id) => {
            let public_key = if let Ok(conn) = &mut db_pool.get()
                && let Ok(user) = crate::schema::users::dsl::users
                    .filter(crate::schema::users::dsl::id.eq(user_id))
                    .first::<User>(conn)
                && let Ok(public_key) = user.public_key()
            {
                public_key
            } else {
                return Ok(HttpResponse::InternalServerError().body("Database error"));
            };
            let verified = check_user_entry(user_id, &entry, &db_pool);
            if verified.is_err() {
                return Ok(HttpResponse::InternalServerError().body("Database error"));
            }
            if !verified.unwrap() {
                return Ok(HttpResponse::Unauthorized().body("Invalid entry"));
            }
            let file_dir = config
                .save_path
                .join(user_id.to_string())
                .join(entry.to_string());
            if !file_dir.exists() {
                std::fs::create_dir_all(&file_dir)?;
            }

            let NewUpdate { aes_key, signature } =
                if let Some(Ok(mut metadata_field)) = payload.next().await {
                    if !metadata_field
                        .content_disposition()
                        .unwrap()
                        .get_name()
                        .unwrap_or("")
                        .starts_with("metadata")
                    {
                        return Ok(HttpResponse::BadRequest().body("Invalid multipart form"));
                    } else {
                        if let Ok(Ok(metadata)) = metadata_field.bytes(2000).await
                            && let Ok(update_info) = postcard::from_bytes::<NewUpdate>(&metadata)
                        {
                            update_info
                        } else {
                            return Ok(HttpResponse::BadRequest().body("Invalid metadata"));
                        }
                    }
                } else {
                    return Ok(HttpResponse::BadRequest().body("Invalid multipart form"));
                };

            if let Some(Ok(mut file_field)) = payload.next().await {
                if !file_field
                    .content_disposition()
                    .unwrap()
                    .get_name()
                    .unwrap_or("")
                    .starts_with("file")
                {
                    return Ok(HttpResponse::BadRequest().body("Invalid multipart form"));
                }

                let update_uuid = Uuid::new_v4();
                let file_path = file_dir.join(update_uuid.to_string());
                let mut file = File::create(file_path)?;
                let mut hasher = sha2::Sha256::new();

                while let Some(chunk) = file_field.next().await {
                    let data = chunk?;
                    file.write_all(&data)?;
                    hasher.update(data);
                }

                let hash = hasher.finalize().to_vec();
                if let Ok(result) = common::crypto::verify_signature(&public_key, &hash, &signature)
                    && result
                {
                    let update = Update::new(update_uuid, entry.clone(), aes_key, signature);
                    if let Ok(conn) = &mut db_pool.get()
                        && let Ok(_) = insert_into(crate::schema::updates::dsl::updates)
                            .values(&update)
                            .execute(conn)
                    {
                    } else {
                        return Ok(HttpResponse::InternalServerError().body("Database error"));
                    }
                } else {
                    return Ok(HttpResponse::BadRequest().body("Invalid signature"));
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
    db_pool: Data<DbPool>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    match req.extensions().get::<Uuid>() {
        Some(user_id) => {
            let verified = check_user_entry(user_id, &entry, &db_pool);
            if verified.is_err() {
                return Ok(HttpResponse::InternalServerError().body("Database error"));
            }
            if !verified.unwrap() {
                return Ok(HttpResponse::Unauthorized().body("Invalid entry"));
            }
            use crate::schema::updates::dsl::*;
            let update = if let Ok(conn) = &mut db_pool.get() {
                match updates
                    .filter(entry_id.eq(entry.clone()))
                    .order_by(created.desc())
                    .first::<Update>(conn)
                {
                    Ok(update) => update,
                    Err(_) => return Ok(HttpResponse::NotFound().body("Empty entry")),
                }
            } else {
                return Ok(HttpResponse::InternalServerError().body("Database error"));
            };

            let file_path = config
                .save_path
                .join(user_id.to_string())
                .join(entry.to_string())
                .join(update.id.to_string());
            let mut file = File::open(&file_path).expect("Can't open file!");

            // 读取文件内容
            let mut chunk = vec![];
            file.read_to_end(&mut chunk)?;

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

            Ok(builder.body(chunk))
        }
        None => Ok(HttpResponse::Unauthorized().body("Invalid authorization token")),
    }
}

#[post("entry")]
async fn create_entry(db_pool: Data<DbPool>, req: HttpRequest) -> impl Responder {
    match req.extensions().get::<Uuid>() {
        Some(&user_id) => {
            if let Ok(conn) = &mut db_pool.get() {
                let uuid = Uuid::new_v4();
                match insert_into(crate::schema::entries::dsl::entries)
                    .values(Entry { id: uuid, user_id })
                    .execute(conn)
                {
                    Ok(_) => HttpResponse::Ok().json(common::models::EntryInfo { uuid }),
                    Err(_) => HttpResponse::InternalServerError().body("Database error"),
                }
            } else {
                HttpResponse::InternalServerError().body("Database error")
            }
        }
        None => HttpResponse::Unauthorized().body("Invalid authorization token"),
    }
}

fn check_user_entry(
    user_id: &Uuid,
    entry_id: &Uuid,
    db_pool: &Data<DbPool>,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::schema::entries::dsl;
    let conn = &mut db_pool.get()?;
    let count = dsl::entries
        .filter(dsl::user_id.eq(user_id).and(dsl::id.eq(entry_id)))
        .count()
        .get_result::<i64>(conn)?;
    Ok(count > 0)
}
