#[macro_use]
extern crate rocket;

// TODO: 快速测试
#[allow(unused)]
type Result<T> = std::result::Result<T, String>;

/// 检查日期是否正确
///
/// - 检查月份是否存在: [1, 12]
fn check_date(month: u8, year: i32) -> Result<()> {
	if month < 1 || month > 12 {
		return Err(format!("`{month}` is not an available month."));
	}

	// TODO: 检查年份
	let _year = year;

	Ok(())
}

/// 检查时间偏移量是否允许
///
/// 偏移量大小应该小于等于 12hours, 允许正负
fn check_offset(offset: i8) -> Result<()> {
	if offset < -12 || offset > 12 {
		return Err(format!("`{offset}` is not a allowed offset."));
	}

	Ok(())
}

#[get("/dates?<month>&<year>")]
fn dates(month: u8, year: i32) -> Result<String> {
	check_date(month, year)?;
	Ok(format!("{year}-{month:02}"))
}

#[get("/fixed_dates?<offset>&<month>&<year>")]
fn fixed_dates(offset: i8, month: u8, year: i32) -> Result<String> {
	check_date(month, year)?;
	check_offset(offset)?;
	Ok(format!("{year}-{month:02} [{offset:+02}]"))
}


#[launch]
fn rocket() -> _ {
	rocket::build().mount("/stat", routes![dates, fixed_dates])
}
