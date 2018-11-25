// Standard library includes
use std::env;

// Library includes
use mysql as my;

fn get_opts() -> my::Opts {
	let mut builder = my::OptsBuilder::new();
	builder
	    .ip_or_hostname(env::var("DB_HOST").ok())
	    .db_name(env::var("DB_DATABASE").ok())
	    .user(env::var("DB_USER").ok())
	    .pass(env::var("DB_PASS").ok());
	
	builder.into()
}

pub fn get_connection() -> my::Pool {
	let options = get_opts();
	my::Pool::new(options).unwrap()
}