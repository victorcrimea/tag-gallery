use chrono::Duration;
use lambda_web::{is_running_on_lambda, launch_rocket_on_lambda, LambdaError};
use pool_async::{ImageProcessorPoolAsync, Job, Settings};
use rocket::data::{Limits, ToByteUnit};
use rocket::tokio::sync::mpsc;
use rocket::tokio::task;
use rocket::tokio::time;
use rocket::Rocket;
use rocket::{routes, Build};
use rocket_async_compression::Compression;
use rocket_db_pools::sqlx;
use rocket_db_pools::Database;
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::okapi::schemars::gen::SchemaSettings;
use rocket_okapi::settings::UrlObject;
use rocket_okapi::{mount_endpoints_and_merged_docs, rapidoc::*};
use tokio::sync::mpsc::Sender;

//use crate::pool_async::{ImageProcessorPoolAsync, Settings};

mod endpoints;
mod error;
mod pool_async;
//mod image_processor_pool;
//Request handlers
//mod crawler;
//mod healthcheck;
//mod image;
//mod image_processor;

// Local includes
//use image_processor_pool::{ImageProcessorPool, ImageProcessorPoolShared};

// Type aliases
pub type ApiResult<T> = Result<rocket::serde::json::Json<T>, error::ApiError>;
pub type DataResult<'a, T> = Result<rocket::serde::json::Json<T>, rocket::serde::json::Error<'a>>;

// fn login_handler(request: &mut Request) -> IronResult<Response> {
//     println!("{:?}", request.get_ref::<Params>());

//     let response = "login".to_string(); // + name;

//     Ok(Response::with((status::Ok, response)))
// }

#[derive(Database)]
#[database("tag_gallery")]
pub struct MainDB(sqlx::MySqlPool);

pub struct TestState(mpsc::Sender<Job>);

async fn start_processor() -> Sender<Job> {
	let pool = sqlx::MySqlPool::connect("mysql://root:helloworld@local-db/tag_gallery")
		.await
		.unwrap();

	let processor_sender = ImageProcessorPoolAsync::new(
		pool,
		Settings {
			gallery_folder: "/home/victorcrimea/tag_gallery".to_string(),
		},
	)
	.await
	.sender();

	processor_sender
}

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
	let tx = start_processor().await;
	let rocket_app = create_server(tx).attach(Compression::fairing());

	// if is_running_on_lambda() {
	// 	// Launch on AWS Lambda
	// 	println!("Running on lambda!");
	// 	launch_rocket_on_lambda(rocket_app).await?;
	// } else {
	// 	// Launch local server
	// 	println!("Running on directly");
	// 	let _ = rocket_app.launch().await?;
	// }

	// Launch local server
	println!("Running directly");
	let _ = rocket_app.launch().await?;
	println!("PRINT_TEST");

	Ok(())
}

pub fn create_server(tx: mpsc::Sender<Job>) -> Rocket<Build> {
	let figment = rocket::Config::figment().merge((
		"limits",
		Limits::new()
			.limit("data-form", 6.mebibytes())
			.limit("file", 4.mebibytes()),
	));

	let mut rocket_app = rocket::custom(&figment)
		.mount("/", routes![endpoints::tester::add_data])
		.mount(
			"/docs/",
			make_rapidoc(&RapiDocConfig {
				title: Some("Tag Gallery documentation".to_owned()),
				general: GeneralConfig {
					spec_urls: vec![UrlObject::new("Rust", "openapi.json")],
					..Default::default()
				},
				ui: UiConfig {
					theme: Theme::Dark,
					..Default::default()
				},
				layout: LayoutConfig {
					render_style: RenderStyle::Focused,
					..Default::default()
				},
				hide_show: HideShowConfig {
					allow_spec_url_load: false,
					allow_spec_file_load: false,
					show_header: false,
					..Default::default()
				},
				custom_html: Some(String::from(include_str!("../static/rapidoc.html"))),
				..Default::default()
			}),
		)
		.attach(MainDB::init())
		.manage(TestState(tx));

	let schema_settings = SchemaSettings::openapi3().with(|s| {
		s.option_nullable = false;
	});

	let openapi_settings = rocket_okapi::settings::OpenApiSettings {
		schema_settings,
		json_path: "/docs/openapi.json".to_owned(),
	};
	let custom_route_spec = (vec![], custom_openapi_spec());

	mount_endpoints_and_merged_docs! {
		rocket_app, "/", openapi_settings,
		"/__DOCS_ONLY__" => custom_route_spec,
		"/api" => endpoints::crawler::get_routes_and_docs(&openapi_settings),
		"/api" => endpoints::healthcheck::get_routes_and_docs(&openapi_settings),
		"/api" => endpoints::processor::get_routes_and_docs(&openapi_settings),
		"/api" => endpoints::auth::get_routes_and_docs(&openapi_settings),
		"/api" => endpoints::image::get_routes_and_docs(&openapi_settings),
		//"/api" => endpoints::tester::get_routes_and_docs(&openapi_settings),
	};

	// CORS settings

	if let Ok(cors) = rocket_cors::CorsOptions::default().to_cors() {
		rocket_app = rocket_app.attach(cors)
	}

	rocket_app
}

fn custom_openapi_spec() -> OpenApi {
	use rocket_okapi::okapi::openapi3::*;

	OpenApi {
		openapi: OpenApi::default_version(),
		info: Info {
			title: "Tag gallery API".to_owned(),
			contact: Some(Contact {
				name: Some("Victor".to_owned()),
				url: Some("mailto:suit.uanic@gmail.com".to_owned()),
				email: None,
				..Default::default()
			}),
			version: "0.1.0".to_owned(),
			..Default::default()
		},
		servers: vec![Server {
			url: "http://127.0.0.1:8000".to_owned(),
			description: Some("Localhost".to_owned()),
			..Default::default()
		}],
		..Default::default()
	}
}

// fn main() {
// 	env_logger::init();

// 	let mut settings = config::Config::default();
// 	settings
// 		// Add in settings from the settings.toml file
// 		.merge(config::File::with_name("settings"))
// 		.unwrap_or(&mut config::Config::default());

// 	let settings = settings.try_into::<HashMap<String, String>>().unwrap_or_default();
// 	println!(
// 		"Running with config: \n{:?}",
// 		settings
// 	);

// 	//Create router instance
// 	let mut router = Router::new();
// 	router.post("/api/add_source_path",
// 		crawler::add_source_path,
// 		"add_source_path"
// 	);
// 	router.get("/api/list_source_paths",
// 		crawler::list_source_paths,
// 		"list_source_paths"
// 	);

// 	router.get("/api/list_photos/:id",
// 		crawler::list_photos,
// 		"list_photos"
// 	);
// 	router.get("/api/healthcheck",
// 		healthcheck::get_handler,
// 		"healthcheck"
// 	);
// 	router.post("/api/login/:name",
// 		login_handler,
// 		"login"
// 	);
// 	router.post("/api/process_source_path",
// 		image_processor::process_source_path,
// 		"process_source_path"
// 	);
// 	router.get("/api/process_status",
// 		image_processor::process_status,
// 		"process_status"
// 	);
// 	router.get("/api/image/:id/:size",
// 		image::get,
// 		"get_image"
// 	);

// 	let mut chain = Chain::new(router);
// 	let (logger_before, logger_after) = Logger::new(None);
// 	chain.link_before(logger_before);

// 	chain.link_after(logger_after);

// 	// Initialize shared image processor pool
// 	let image_processor_pool = ImageProcessorPool::new(settings.clone());

// 	// Persistent data
// 	chain.link_before(
// 		State::<ImageProcessorPoolShared>::one(image_processor_pool)
// 	);

// 	chain.link_before(
// 		State::<Settings>::one(settings)
// 	);

// 	let bind = "0.0.0.0:3000";

// }
