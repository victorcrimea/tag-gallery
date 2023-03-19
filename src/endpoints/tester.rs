use rand::random;
use rocket::get;
use rocket::State;
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::openapi;
use rocket_okapi::openapi_get_routes_spec;
use rocket_okapi::settings::OpenApiSettings;

use crate::pool_async::Job;

// pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
// 	openapi_get_routes_spec![settings: add_data]
// }

//#[openapi(tag = "System")]
#[get("/add_data")]
pub async fn add_data(test_state: &State<crate::TestState>) -> String {
	//let r: u8 = random();
	// for _ in 0..5 {
	// 	test_state
	// 		.0
	// 		.send("TEST: ".to_string() + r.to_string().as_str())
	// 		.await
	// 		.unwrap();
	// }

	test_state.0.send(Job { source_id: 1 }).await.unwrap();

	String::from("Rocket health is good too!")
}
