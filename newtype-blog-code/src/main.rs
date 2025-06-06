// codigo del blog: https://www.howtocodeit.com/articles/ultimate-guide-rust-newtypes
// El paradigma newtype se explica en el libro oficial de Rust para evitar caer en lo que se conoce
// como el "orphan rule": que es cuando queremos impl un trait para un type pero el lenguaje nos
// impone que o el trait o el type esten definidos en nuestro crate
// Como ejemplo nos daban un `Vec<String>` que queriamos que impl Display entonces haciamos un
// wrapper:

use bcrypt::BcryptError;
use core::{fmt, panic};
use std::{fmt::Display, ops::Sub, sync::atomic::Ordering};
use thiserror::Error;

struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

// pero el uso realmente importante es cuando queremos impl el paradigma de type-driven que hace
// que sea casi imposible ingresar datos que no son correctos
//
// el ejemplo clasico es cuando necesitamos el email y password:
struct UserWrong {
    email: String,
    password: String,
}

fn validate_password_wrong(input: &str) -> Result<(), CreateUserError> {
    todo!()
}

fn validate_email_wrong(input: &str) -> Result<(), CreateUserError> {
    todo!()
}

fn hash_password_wrong(input: &str) -> Result<(), CreateUserError> {
    todo!()
}

#[derive(Error, Debug)]
pub enum CreateUserError {
    #[error("invalid email address: {0}")]
    InvalidEmail(String),
    #[error("Invalid password: {reason}")]
    InvalidPassword { reason: String },
    #[error("failed to hash password")]
    PasswordHashError(#[from] BcryptError),
    #[error("a user with this email address {0:?}, already exists!!!")]
    UserAlreadyExistsError(EmailAddress),
}

fn create_user_wrong(email: &str, password: &str) -> Result<UserWrong, CreateUserError> {
    validate_email_wrong(email)?;
    validate_password_wrong(password)?;

    let password_hash = hash_password_wrong(password)?;
    // ...
    let user = UserWrong {
        email: email.to_owned(),
        password: password.to_owned(),
    };

    Ok(user)
}

// Uno de los principales problemas de esto es que deja lugar a muchos inputs posibles y no pueden
// ser testeados todas las permutaciones de caso de fallo...
//
// Newtyping es la pratica de invertir tiempo en el diseño de tipos de datos que son siempre
// validos. A largo plazo prevenimos error humano...
/// Esenciales del newtyping
use validator::ValidateEmail;

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    email: EmailAddress,
    password: Password,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(raw_email: &str) -> Result<Self, EmailAddressError> {
        if raw_email.validate_email() {
            Ok(Self(raw_email.into()))
        } else {
            Err(EmailAddressError(raw_email.into()))
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
#[error("a user with this email address {0:?}, already exists!!!")]
pub struct UserAlreadyExistsError(EmailAddress);

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0} is not a valid email address!!!")]
pub struct EmailAddressError(String);

#[derive(Debug, Clone, PartialEq)]
pub struct Password(String);

pub fn validate_email(email: &EmailAddress) -> Result<(), CreateUserError> {
    todo!()
}

pub fn validate_password(password: &Password) -> Result<(), CreateUserError> {
    todo!()
}

pub fn hash_password(password: &Password) -> Result<(), CreateUserError> {
    todo!()
}

pub fn create_user(email: EmailAddress, password: Password) -> Result<User, CreateUserError> {
    validate_email(&email)?;
    validate_password(&password)?;
    let password_hash = hash_password(&password)?;

    //...
    let user = User { email, password };
    Ok(user)
}

// Un error comun seria poner un `pub` dentro de la tuple struct para poder acceder a el valor que
// estamos haciendo el wrapper
//
// Los constructores son la fuente de verdad

/// mutabilidad en newtypes
// en algunos casos tiene sentido que el newtype pueda ser mutable, pero tenemos que tener cuidado
// que preserve sus invariants

struct NonEmptyVector<T>(Vec<T>);

impl<T> NonEmptyVector<T> {
    fn pop(&mut self) -> Option<T> {
        if self.0.len() == 1 {
            None
        } else {
            self.0.pop()
        }
    }

    fn last(&self) -> &T {
        self.0.last().unwrap()
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// implementando casos especiales manualmente
// Por ejemplo para el siguiente ejemplo donde impl `Subsecond` y `f64` que representa una fraccion
// de un segundo en el rango 0.0..1.0
#[derive(Error, Debug, Clone, Copy, PartialEq)]
pub struct Subsecond(f64);

impl Display for Subsecond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Subsecond {
    pub fn new(raw: f64) -> Result<Self, SubsecondError> {
        if !(0.0..1.0).contains(&raw) {
            Err(SubsecondError(raw))
        } else {
            Ok(Self(raw))
        }
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq)]
#[error("Subsecond value must be in the range 0.0..1.0 but was {0}")]
pub struct SubsecondError(f64);

// Como sabemos `f64` no puede impl o `Eq` o `Ord` dado que `f64::NAN` no es igual a nada ni
// siquiera a si mismo!!!. Felizmente esto no es asi para `Subsecond` Existe una relacion de
// igualdad para todos los `f64` que estan en el rango de 0.0..1.0
//
impl Eq for Subsecond {}

impl PartialOrd for Subsecond {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Subsecond {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.0.partial_cmp(&other.0) {
            Some(ordering) => ordering,
            None => unreachable!(),
        }
    }
}

// Escribiendo constructores de newtypes ergonomicos con `From` y `TryFrom`
// Elegimos `From` cuando la conversion es infalible y `TryFrom` cuando puede fallar
// Por ejemplo:
struct WrappedI32(i32);

impl From<i32> for WrappedI32 {
    fn from(raw: i32) -> Self {
        Self(raw)
    }
}

// Desde newtypes a types primitivos con `AsRef` `Deref` `Borrow`
//
// Como deberiamos obtener el type que esta debajo de un newtype??? Yo no se sobre el client de
// nuestra base de datos pero la mia acepta solo `&str` y no `EmailAddress`
//
// Por ello lo que queremos es hacer que los getters sean mas amigables por ejemplo:

impl EmailAddress {
    pub fn into_string(self) -> String {
        self.0
    }
}

fn main() {
    println!("Hello, world!");
}
