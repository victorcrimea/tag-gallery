use rocket::get;
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::openapi;
use rocket_okapi::openapi_get_routes_spec;
use rocket_okapi::settings::OpenApiSettings;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_healthcheck]
}

/// # Get health status of Rocket app
///
/// Returns fixed string if server is running
#[openapi(tag = "System")]
#[get("/healthcheck-rs")]
async fn get_healthcheck() -> String {
    String::from("Rocket health is good too!")
}
