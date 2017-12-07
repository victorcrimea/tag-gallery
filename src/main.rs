extern crate iron; 
use iron::prelude::*;
use iron::status;

fn echo(request: &mut Request) -> IronResult<Response> {
	let body = "OUT".to_string();
	Ok(Response::with((status::Ok, body)))
}

fn main() {
	Iron::new(echo).http("localhost:3000").unwrap();
}