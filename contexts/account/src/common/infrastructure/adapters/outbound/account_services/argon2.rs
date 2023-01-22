use crate::{common::application::ports::outbound::account_services};

use std::fmt::Debug;

use anyhow::anyhow;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

pub struct AccountServices<'a> {
    argon: Argon2<'a>,
}

impl<'a> AccountServices<'a> {
    pub fn new() -> Self {
        return Self {
            argon: Argon2::default(),
        };
    }
}

impl<'a> Debug for AccountServices<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountServices")
            .field("argon", &"Argon2")
            .finish()
    }
}

impl<'a> account_services::TAccountServices for AccountServices<'a> {
    fn hash_password(&self, password: String) -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        match self.argon.hash_password(password.as_bytes(), &salt) {
            Ok(x) => Ok(x.to_string()),
            Err(e) => return Err(anyhow!(e)),
        }
    }
}

impl<'a> account_services::AccountServices for AccountServices<'a> {}