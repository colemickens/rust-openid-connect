/// Derived from iron_login: https://github.com/untitaker/iron-login

use iron::prelude::*;
use iron::middleware;
use iron::typemap::Key;
use iron::modifier;
use iron_sessionstorage::traits::*;
use iron_sessionstorage::SessionStorage;
use iron_sessionstorage::backends::SignedCookieBackend;
use persistent;
use cookie::Cookie;
use result::Result;

#[derive(Clone)]
pub struct LoginManager {
    signing_key: Vec<u8>,
    /// Configuration for this manager
    pub config: LoginConfig,
}

impl LoginManager {
    /// Construct a new login middleware using the provided signing key
    pub fn new(signing_key: Vec<u8>) -> LoginManager {
        LoginManager {
            signing_key: signing_key,
            config: LoginConfig::defaults(),
        }
    }
}

/// Configuration
#[derive(Debug, Clone)]
pub struct LoginConfig {
    /// This cookie contains the default values that will be used for session cookies.
    ///
    /// You may e.g. override `httponly` or `secure` however you wish.
    pub cookie_base: Cookie<'static>,
}

impl LoginConfig {
    /// Construct a configuration instance with default values
    pub fn defaults() -> Self {
        LoginConfig {
            cookie_base: Cookie::build("logged_in_user".to_owned(), "".to_owned())
                    .http_only(true)
                    .path("/")
                    .secure(true)
                    .finish()
        }
    }
    
    pub fn get_config(req: &mut Request) -> Result<LoginConfig> {
        let login_config = (*try!(req.get::<persistent::Read<LoginConfig>>())).clone();
        
        Ok(login_config)
    }
}

impl Key for LoginConfig { type Value = LoginConfig; }

impl middleware::AroundMiddleware for LoginManager {
    fn around(self, handler: Box<middleware::Handler>) -> Box<middleware::Handler> {
        let mut ch = Chain::new(handler);
        let key = self.signing_key;

        ch.link_around(SessionStorage::new(SignedCookieBackend::new(key)));
        ch.link(persistent::Read::<LoginConfig>::both(self.config));

        Box::new(ch)
    }
}

/*pub struct LoginModifier<U: LoginSession> {
    login: Login<U>
}

impl <U: LoginSession> modifier::Modifier<Response> for LoginModifier<U> {
    fn modify(self, response: &mut Response) {
        response.set_cookie({
            let mut x = self.login.config.cookie_base.clone();
            x.value = self.login.session.map_or_else(|| "".to_owned(), |u| u.get_id().unwrap_or(String::new()));
            x
        });
    }
}*/

#[derive(Clone, Debug)]
pub struct Login<U: LoginSession> {
    pub session: Option<U>,
    pub config: LoginConfig,
}

impl <U: LoginSession> Login<U> {
    pub fn new(config: &LoginConfig, session: Option<U>) -> Login<U> {
        Login {
            session: session,
            config: config.clone(),
        }
    }
    
    /*pub fn cookie(&self) -> LoginModifier<U> {
        LoginModifier {
            login: (*self).clone()
        }
    }*/
}

pub trait LoginSession: Clone + Send + Sync + Sized {
    fn get_id(&self) -> Option<String>;
}