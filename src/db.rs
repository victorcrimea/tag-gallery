use mysql as my;

// pub fn db_test() {
// 	let pool = my::Pool::new("mysql://root:getitstarted@localhost:3306").unwrap();

// 	pool.prep_exec(r"CREATE TABLE tag_gallery.payment (
// 					 customer_id int not null,
// 					 amount int not null,
// 					 account_name text
// 					 )", ()).unwrap();
// }


fn get_opts() -> my::Opts {
	let mut builder = my::OptsBuilder::new();
	builder
	    .ip_or_hostname(Some("localhost".to_string()))
	    .db_name(Some("tag_gallery".to_string()))
	    .user(Some("root".to_string()))
	    .pass(Some("".to_string()));
	
	builder.into()
}

pub fn get_connection() -> my::Pool {
	
	let options = get_opts();

	my::Pool::new(options).unwrap()
}