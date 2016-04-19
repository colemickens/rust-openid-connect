use std::result;
use result::{Result, OpenIdConnectError};
use users::{User, UserRepo};
/// A way of authenticating users against a repository of users.

pub enum AuthenticationStatus {
    UserNotFound,
    IncorrectPassword,
    Success,
    //Continue, second factor?
}

pub trait Authenticator {
    fn authenticate(&self, username: &str, password: &str) -> Result<AuthenticationStatus>;
}


