use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

use crate::errors::ServiceError;

pub fn hash_password(password: &str) -> Result<String, ServiceError> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string();

    Ok(hash)
}

pub fn verify(hash: &str, password: &str) -> Result<(), ServiceError> {
    let argon2 = Argon2::default();

    let hash = PasswordHash::new(hash).map_err(|_| ServiceError::Unauthorized)?;
    argon2
        .verify_password(password.as_bytes(), &hash)
        .map_err(|_| ServiceError::Unauthorized)
}
