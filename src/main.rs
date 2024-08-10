#[macro_use]
extern crate rocket;

#[get("/dates?<month>&<year>")]
fn dates(month: u8, year: i32) -> String {
	format!("{year}-{month:02}")
}

#[get("/fixed_dates?<offset>&<month>&<year>")]
fn fixed_dates(offset: i8, month: u8, year: i32) -> String {
	format!("{year}-{month:02} [{offset:+02}]")
}


#[launch]
fn rocket() -> _ {
	rocket::build().mount("/stat", routes![dates, fixed_dates])
}
