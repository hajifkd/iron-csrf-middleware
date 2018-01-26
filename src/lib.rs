extern crate crypto;
extern crate iron;
extern crate iron_sessionstorage;
extern crate params;

use std::error::Error;
use std::fmt::{self, Debug};
use iron::prelude::*;
use iron::{typemap, BeforeMiddleware};
use iron_sessionstorage::traits::*;
use params::{Params, Value};
use crypto::sha2::Sha256;
use crypto::digest::Digest;

/// CSRF token
#[derive(Debug, PartialEq, Eq)]
pub struct CsrfToken(pub String);

pub const QUERY_KEY: &'static str = "_csrf_token";

impl CsrfToken {
    fn new(secret: &str) -> CsrfToken {
        let time = std::time::SystemTime::now();
        let mut hasher = Sha256::new();

        hasher.input_str(&format!("{}{:?}", secret, time));

        CsrfToken(hasher.result_str())
    }
}

impl iron_sessionstorage::Value for CsrfToken {
    fn get_key() -> &'static str {
        "_csrf_token"
    }

    fn into_raw(self) -> String {
        self.0
    }

    fn from_raw(value: String) -> Option<Self> {
        if value.is_empty() {
            None
        } else {
            Some(CsrfToken(value))
        }
    }
}

pub trait CsrfReqExt {
    fn csrf_token(&mut self) -> String;
}

impl<'a, 'b> CsrfReqExt for Request<'a, 'b> {
    fn csrf_token(&mut self) -> String {
        self.session().get::<CsrfToken>().unwrap().unwrap().0
    }
}

/// Iron middleware to check and generate CSRF token.
pub struct CsrfMiddleware {
    secret: String,
}

/// Creates a new instance with the given secret.
impl CsrfMiddleware {
    pub fn new(secret: &str) -> CsrfMiddleware {
        CsrfMiddleware {
            secret: secret.to_owned(),
        }
    }
}

impl typemap::Key for CsrfMiddleware {
    type Value = CsrfToken;
}

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &*self.0
    }
}

impl BeforeMiddleware for CsrfMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        let token = if let Ok(Some(CsrfToken(ref token))) = req.session().get::<CsrfToken>() {
            token.to_owned()
        } else {
            let token = CsrfToken::new(&self.secret);
            let token_str = token.0.clone();
            req.session().set::<CsrfToken>(token).unwrap();
            token_str
        };

        if req.method != iron::method::Method::Post {
            Ok(())
        } else {
            let params = req.get_ref::<Params>().unwrap();
            if let Some(&Value::String(ref user_token)) = params.get(QUERY_KEY) {
                if *user_token == token {
                    Ok(())
                } else {
                    Err(IronError::new(
                        StringError("Bad token".to_owned()),
                        iron::status::BadRequest,
                    ))
                }
            } else {
                Err(IronError::new(
                    StringError("No token".to_owned()),
                    iron::status::BadRequest,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
