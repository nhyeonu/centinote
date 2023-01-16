use actix_web::{Error, error::{ErrorUnauthorized, ErrorInternalServerError}};
use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use super::User;

fn verify_password_argon2(
    password_hash: &str,
    password: &str) -> Result<(), password_hash::errors::Error>
{
    let parsed_hash = PasswordHash::new(password_hash)?;
    Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?;
    Ok(())
}

impl User {
    pub fn verify_password(
        &self,
        password: &str) -> Result<(), Error> 
    {
        match verify_password_argon2(&self.password_hash, password) {
            Ok(value) => Ok(value),
            Err(error) => {
                match error {
                    password_hash::errors::Error::Password => {
                        Err(ErrorUnauthorized("Password is wrong."))
                    },
                    _ => {
                        Err(ErrorInternalServerError("Password verification failed."))
                    }
                }
            }
        }
    }
}
