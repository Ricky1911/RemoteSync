use actix_web::{
    HttpResponse, Responder, post,
    web::{Data, Json},
};
use common::{
    crypto::{bytes_to_public_key, public_key_to_bytes},
    models::NewUser,
};
use diesel::{insert_into, prelude::*};
use rand::RngExt as _;
use rsa::RsaPublicKey;
use sha2::{Digest as _, Sha256};
use uuid::Uuid;

use crate::DbPool;

pub fn hash_with_salt(password: &str, salt: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(salt);
    hasher.finalize().to_vec()
}

#[derive(Insertable, Queryable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub password: Vec<u8>,
    pub salt: String,
    pub public_key: Vec<u8>,
}

impl User {
    pub fn create_user(
        name: &str,
        password: &str,
        public_key: &RsaPublicKey,
    ) -> Result<Self, common::crypto::Error> {
        let salt: String = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(10) // 字符串长度为 10
            .map(char::from)
            .collect();
        let password_hash = hash_with_salt(password, &salt);
        Ok(User {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            password: password_hash,
            salt,
            public_key: public_key_to_bytes(public_key)?,
        })
    }

    pub fn public_key(&self) -> Result<RsaPublicKey, common::crypto::Error> {
        bytes_to_public_key(&self.public_key)
    }

    pub fn set_public_key(&mut self, key: RsaPublicKey) -> Result<(), common::crypto::Error> {
        self.public_key = public_key_to_bytes(&key)?;
        Ok(())
    }
}

impl TryFrom<&NewUser> for User {
    type Error = common::crypto::Error;
    fn try_from(value: &NewUser) -> Result<Self, Self::Error> {
        let NewUser {
            name,
            password,
            public_key,
        } = value;
        User::create_user(name, password, public_key)
    }
}

#[post("user")]
pub async fn post_user(new_user: Json<NewUser>, db_pool: Data<DbPool>) -> impl Responder {
    use crate::schema::users::dsl::*;
    if let Ok(conn) = &mut db_pool.get()
        && let Ok(existed_users) = users
            .filter(name.eq(&new_user.name))
            .count()
            .get_result::<i64>(conn)
    {
        if existed_users > 0 {
            HttpResponse::Conflict().body(format!("User with name {} already exist", new_user.name))
        } else {
            match User::try_from(&new_user.into_inner()) {
                Ok(new_user) => match insert_into(users).values(&new_user).execute(conn) {
                    Ok(_) => HttpResponse::Ok().finish(),
                    Err(_) => HttpResponse::InternalServerError().body("Database error"),
                },
                Err(_) => {
                    HttpResponse::InternalServerError().body("Failed to serialize public key")
                }
            }
        }
    } else {
        HttpResponse::InternalServerError().body("Database error")
    }
}
