use std::collections::HashMap;

use iron::prelude::*;
use iron::status;
use iron::modifiers::Redirect;
use urlencoded::{UrlEncodedBody, UrlEncodedQuery};
use handlebars_iron::Template;

use vlad::result::{VladError};
use vlad::state::*;
use vlad::params::*;

use result::{Result, OpenIdConnectError};
use urls::*;
use config::Config;
use users::*;

#[derive(Clone, Debug)]
pub struct RegisterRequest {
    username: String,
    password: String
}

#[derive(Clone, Debug)]
pub struct RegisterRequestBuilder {
    username: Option<String>,
    password: Option<String>,
    
    validation_state: ValidationState,
}

impl RegisterRequestBuilder {
    pub fn new() -> RegisterRequestBuilder {
        RegisterRequestBuilder {
            username: None,
            password: None,
            
            validation_state: ValidationState::new(),
        }
    }
    
    pub fn build(self) -> Result<RegisterRequest> {
        if self.validation_state.valid {
            Ok(RegisterRequest {
                username: self.username.unwrap(),
                password: self.password.unwrap(),
            })
        } else {
            Err(OpenIdConnectError::from(VladError::ValidationError(self.validation_state)))
        }
    }
    
    pub fn load_params(&mut self, params: &HashMap<String, Vec<String>>) -> Result<bool> {
        if let Some(username) = try!(multimap_get_maybe_one(params, "username")) {
            self.username = Some(username.to_owned());
        } else {
            self.validation_state.reject("username", VladError::MissingRequiredValue("username".to_owned()));
        }
        
        if let Some(password) = try!(multimap_get_maybe_one(params, "password")) {
            self.password = Some(password.to_owned());
        } else {
            self.validation_state.reject("password", VladError::MissingRequiredValue("password".to_owned()));
        }
        
        Ok(self.validation_state.valid)
    }
    
    pub fn build_from_params(params: &HashMap<String, Vec<String>>) -> Result<RegisterRequest> {
        let mut builder = RegisterRequestBuilder::new();
        
        try!(builder.load_params(params));
        
        builder.build()
    }
}

pub fn new_register_form() -> HashMap<String, String> {
    let mut data = HashMap::new();
    
    data.insert("username".to_owned(), String::new());
    data.insert("password".to_owned(), String::new());
    
    data
}

pub fn register_get_handler(_config: &Config, req: &mut Request) -> IronResult<Response> {
    let mut data = new_register_form();
    
    match req.get_ref::<UrlEncodedQuery>() {
        Ok(params) => {
            debug!("parsed query params: {:?}", params);
            
            match RegisterRequestBuilder::build_from_params(params) {
                Ok(register_request) => {
                    //TODO escape values to protect against cross-site-scripting
                    data.insert("username".to_owned(), register_request.username);
                    data.insert("password".to_owned(), register_request.password);
                },
                Err(err) => {
                    debug!("invalid registration details: {:?}", err);
                }
            }
        },
        Err(err) => {
            debug!("error parsing query params: {:?}", err);
        }
    };
    
    data.insert("_view".to_owned(), "register.html".to_owned());
    
    Ok(Response::with((status::Ok, Template::new("_layout.html", data))))
}

pub fn register_post_handler(config: &Config, req: &mut Request) -> IronResult<Response> {
    let register_url = try!(relative_url(req, "/register"));
    let home_url =try!(relative_url(req, "/"));
    
    match req.get_ref::<UrlEncodedBody>() {
        Ok(params) => {
            debug!("registering new user with creds {:?}", params);
            // TODO validate csrf
            // TODO create session and set cookie
            // TODO multistep registration flow
            // TODO redirect to flow caller
            
            match RegisterRequestBuilder::build_from_params(params) {
                Ok(register_request) => {
                    let user = User::new(register_request.username, Some(register_request.password));
                    
                    debug!("add user to repo: {:?}", user);
                
                    try!(config.user_repo.add_user(user));
                },
                Err(err) => {
                    debug!("user validation errors: {:?}", err);
                }
            }
            
            Ok(Response::with((status::Found, Redirect(home_url))))
        },
        Err(err) => {
            debug!("error parsing body: {:?}", err);
            Ok(Response::with((status::Found, Redirect(register_url))))
        }
    }
}