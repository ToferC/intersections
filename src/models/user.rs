// Modeling from: https://github.com/clifinger/canduma/blob/master/src/user/handler.rs

use chrono::*;
use uuid::Uuid;
use crate::error_handler::CustomError;


#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct User {
    user_id: i32,
    pub user_uuid: Uuid,
    pub hash: Vec<u8>,
    pub salt: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub role: String,
}

#[derive(Debug. Insertable)]
#[table_name = "users"]
pub struct InsertableUser {
    pub user_uuid: Uuid,
    pub hash: Vec<u8>,
    pub salt: String,
    pub created_at: NaiveDateTime,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlimUser {
    pub user_uuid: Uuid,
    pub email: String,
    pub role: String,
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
            name,
            email,
            password,
            ..
        } = user_data;

        let salt = make_salt();
        let hash = make_hash(&password, &salt).to_vec();
        
        Self {
            user_uuid: Uuid::new_v4(),
            email,
            hash,
            created_at: chrono::Local::now().naive_local(),
            salt,
            role: "user".to_owned(),
        }
    }
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        let User {
            user_uuid,
            email,
            role,
            ..
        } = user;

        Self {
            user_uuid,
            email,
            role,
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

fn make_hash(password: &str, salt: &str) -> [u8; argon2rs::defaults::LENGTH] {
    argon2i_simple(password, salt)
}

fn verify(user: &User, password: &str) -> bool {
    let User {hash, salt, ..} = user;

    make_hash(password, salt) == hash.as_ref()
}

fn has_role(user: &LoggedUser, role: &str) -> Result<bool, CustomError> {
    match user.0 {
        Some(ref user) if user.role == role => Ok(true),
        _ => CustomError::new(002, "Role not present".to_string()),
    }
}
