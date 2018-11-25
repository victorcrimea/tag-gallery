// Library includes
use iron::prelude::*;
use iron::status;

pub fn get_handler(_request: &mut Request) -> IronResult<Response> {
	Ok(Response::with((status::Ok, "ok")))
}