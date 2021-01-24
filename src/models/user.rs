// Modeling from: https://github.com/clifinger/canduma/blob/master/src/user/handler.rs

use std::io::prelude;

use argon2::Config;
use uuid::Uuid;
use crate::error_handler::CustomError;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};

use crate::schema::users;

use actix_identity::{Identity, RequestIdentity};

use shrinkwraprs::Shrinkwrap;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::{QueryDsl, BelongingToDsl};

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    user_id: i32,
    pub user_uuid: Uuid,
    pub hash: Vec<u8>,
    pub salt: String,
    pub email: String,
    pub user_name: String,
    pub created_at: NaiveDateTime,
    pub role: String,
    pub managed_communities: Vec<i32>,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct InsertableUser {
    pub user_uuid: Uuid,
    pub hash: Vec<u8>,
    pub salt: String,
    pub created_at: NaiveDateTime,
    pub email: String,
    pub user_name: String,
    pub role: String,
    pub managed_communities: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlimUser {
    pub user_uuid: Uuid,
    pub email: String,
    pub role: String,
    pub managed_communities: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub user_name: String,
    pub email: String,
    pub password: String,
    pub managed_communities: Vec<i32>,
}

#[derive(Shrinkwrap, Clone, Default)]
pub struct LoggedUser(pub Option<SlimUser>);

impl From<SlimUser> for LoggedUser {
    fn from(slim_user: SlimUser) -> Self {
        LoggedUser(Some(slim_user))
    }
}

impl From<UserData> for InsertableUser {
    fn from(user_data: UserData) -> Self {
        let UserData {
            user_name,
            email,
            password,
            managed_communities,
            ..
        } = user_data;

        let salt = make_salt();
        let hash = make_hash(&password, &salt).as_bytes().to_vec();
        
        Self {
            user_name,
            user_uuid: Uuid::new_v4(),
            email,
            hash,
            created_at: chrono::Local::now().naive_local(),
            salt,
            role: "user".to_owned(),
            managed_communities,
        }
    }
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        let User {
            user_uuid,
            email,
            role,
            managed_communities,
            ..
        } = user;

        Self {
            user_uuid,
            email,
            role,
            managed_communities,
        }
    }
}

// Utility Functions
fn make_salt() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 128;
    let mut rng = rand::thread_rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    password
}

fn make_hash(password: &str, salt: &str) -> String {
    let config = argon2::Config::default();
    argon2::hash_encoded(password.as_bytes(), salt.as_bytes(), &config).unwrap()
}

fn verify(user: &User, password: &str) -> bool {
    let User {hash, salt, ..} = user;

    make_hash(password, salt).as_bytes().to_vec() == *hash
}

fn has_role(user: &LoggedUser, role: &str) -> Result<bool, CustomError> {
    match user.0 {
        Some(ref user) if user.role == role => Ok(true),
        _ => Err(CustomError::new(002, "Role not present".to_string())),
    }
}
