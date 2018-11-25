// Library includes
use params::Params;
use iron::prelude::*;
use iron::status;

// This handler serves image of requested size
pub fn get(request: &mut Request) -> IronResult<Response> {
	let _params = request.get::<Params>().unwrap();

	Ok(
		Response::with(
			(status::Ok, "")
		)
	)
}