#![deny(unused_extern_crates)]

extern crate iron; 
extern crate params;
extern crate router;
extern crate logger;
#[macro_use] extern crate mysql;
extern crate env_logger;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate walkdir;
extern crate rayon;
extern crate image;
extern crate config;
extern crate persistent;

use iron::prelude::*;
use iron::status;
use router::Router;
use logger::Logger;
use params::Params;
use persistent::State;
use std::collections::HashMap;

//DB connectivity
mod db;

mod image_processor_pool;

//Request handlers
mod healthcheck;
mod crawler;
mod image_processor;

use image_processor_pool::{ImageProcessorPool, ImageProcessorPoolShared};

fn login_handler(request: &mut Request) -> IronResult<Response> {
	println!("{:?}", request.get_ref::<Params>());

	let response = "login".to_string();// + name;

	Ok(Response::with((status::Ok, response)))
}

fn main() {
	env_logger::init()


	let mut settings = config::Config::default();
	settings
		// Add in settings from the settings.toml file
		.merge(config::File::with_name("settings"))
		.unwrap_or(&mut config::Config::default());

	let settings = settings.try_into::<HashMap<String, String>>().unwrap_or_default();
	println!(
		"Running with config: \n{:?}",
		settings
	);

	//Create router instance
	let mut router = Router::new();
	router.post("/api/add_source_path",
		crawler::add_source_path,
		"add_source_path"
	);
	router.get("/api/list_source_paths",
		crawler::list_source_paths,
		"list_source_paths"
	);
	router.get("/api/healthcheck",
		healthcheck::get_handler,
		"healthcheck"
	);
	router.post("/api/login/:name",
		login_handler,
		"login"
	);
	router.post("/api/process_source_path",
		image_processor::process_source_path,
		"process_source_path"
	);
	router.get("/api/process_status",
		image_processor::process_status,
		"process_status"
	);

	let mut chain = Chain::new(router);
	let (logger_before, logger_after) = Logger::new(None);
	chain.link_before(logger_before);
	
	chain.link_after(logger_after);

	// Shared state init
	let image_processor_pool = ImageProcessorPool::new(settings);

	//3 is a hypothetical source_id
	// processor_pool.add_source_to_process(3);
	// println!("Status: {:?}", processor_pool.status_of(3));
	// thread::sleep(Duration::from_secs(1));
	// println!("Status: {:?}", processor_pool.status_of(3));

	chain.link_before(
		State::<ImageProcessorPoolShared>::one(image_processor_pool)
	);

	let bind = "0.0.0.0:3000";
	match Iron::new(chain).http(bind) {
		Ok(_) => println!("Server bound to {:?}", bind),
		Err(_) => println!("Server couldn't bind")
	};
}