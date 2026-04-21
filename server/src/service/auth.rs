use actix_web::{HttpMessage as _, HttpRequest, HttpResponse, Responder, post, web};
use common::models::{LoginRequest, TokenResponse};
use diesel::prelude::*;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::DbPool;

const SECRET_KEY: &[u8] = b"your-secret-key"; // 应该从环境变量中读取

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
}

pub fn generate_token(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize
        + 3600;

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY),
    )
}

pub fn verify_token(token: &str) -> Result<Uuid, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::default(),
    )?;
    Ok(token_data.claims.sub)
}

#[post("login")]
pub async fn login(
    login_req: web::Json<LoginRequest>,
    db_pool: web::Data<DbPool>,
) -> impl Responder {
    use crate::schema::users::dsl::*;

    let conn = &mut db_pool.get().unwrap();
    let user_result = users
        .filter(name.eq(&login_req.name))
        .first::<crate::service::user::User>(conn);

    match user_result {
        Ok(user) => {
            let password_hash =
                crate::service::user::hash_with_salt(&login_req.password, &user.salt);
            if password_hash == user.password {
                match generate_token(user.id) {
                    Ok(token) => HttpResponse::Ok().json(TokenResponse { token }),
                    Err(_) => HttpResponse::InternalServerError().body("Failed to generate token"),
                }
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}

pub fn get_user_uuid(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    if let Some(&user_id) = req.extensions().get::<Uuid>() {
        Ok(user_id)
    } else {
        Err(HttpResponse::Unauthorized().body("Invalid authorization token"))
    }
}
