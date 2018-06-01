use mysql as my;
use std::env;
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