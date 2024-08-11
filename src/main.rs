#[macro_use]
extern crate rocket;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket_db_pools::{Connection, Database};
use webrecord::error::*;
use webrecord::func::*;


#[get("/dates?<month>&<year>")]
async fn dates(month: u8, year: i32, db: Connection<Records>) -> (Status, Json<MonthCount>) {
	let mut ans: MonthCount = Vec::with_capacity(31);
	let res = select_dates(&mut ans, month, year, db).await;
	(err_status(res), Json(ans))
}

#[get("/fixed_dates?<offset>&<month>&<year>")]
async fn fixed_dates(offset: i8, month: u8, year: i32, db: Connection<Records>)
                     -> (Status, Json<MonthCount>) {
	let mut ans: MonthCount = Vec::with_capacity(31);
	let res = select_fixed_dates(&mut ans, offset, month, year, db).await;
	(err_status(res), Json(ans))
}


#[launch]
fn rocket() -> _ {
	rocket::build().attach(Records::init())
	               .mount("/stat", routes![dates, fixed_dates])
}
