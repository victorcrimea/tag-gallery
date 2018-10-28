pub fn login(request: &mut Request) -> IronResult<Response> {
	/// Checks provided credentials and if correct generates access token.
	// TODO: Victor Semenov: implement logic

	Ok(Response::with((status::Ok, "ok")))
}

pub fn set_password(request: &mut Request) -> IronResult<Response> {
	/// Allows to set user password if not set yet

	Ok(Response::with((status::Ok, "ok")))
}