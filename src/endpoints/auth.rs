use rocket::State;
use rocket::{get, post, put, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::openapi;
use rocket_okapi::openapi_get_routes_spec;
use rocket_okapi::settings::OpenApiSettings;
use serde::{Deserialize, Serialize};

use crate::MainDB;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: login, set_password]
}

/// Checks provided credentials and if correct generates access token.
///
/// TODO: Victor Semenov: implement logic
#[openapi(tag = "Auth")]
#[post("/login/<name>")]
pub fn login(mut db: Connection<MainDB>, name: &str) -> crate::ApiResult<()> {
    todo!()
}

/// Allows to set user password if not set yet
#[openapi(tag = "Auth")]
#[put("/login/set_password")]
pub fn set_password(mut db: Connection<MainDB>) -> crate::ApiResult<()> {
    todo!()
}
