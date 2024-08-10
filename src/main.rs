#[macro_use]
extern crate rocket;

use rocket::futures::TryStreamExt;
use rocket::serde::{Serialize, json::Json};
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Connection, Database};


#[derive(Database)]
#[database("records")]
struct Records(sqlx::SqlitePool);


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

/// 某日的次数统计
#[derive(Serialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(crate = "rocket::serde")]
struct DayCount {
	/// 日期
	///
	/// 范围为 1 到 (28|29|30|31), 依据请求的月份决定
	///
	/// # Note
	///
	/// 数据库内不会检查日期的可靠性, 只能依靠插入时的检查.
	/// 因此查询时可能得到错误的结果.
	day: u8,

	/// 次数
	times: u32,
}

type MonthCount = Vec<DayCount>;

#[get("/dates?<month>&<year>")]
async fn dates(month: u8, year: i32, mut db: Connection<Records>) -> Result<Json<MonthCount>> {
	check_date(month, year)?;

	const SQL_SELECT: &str =
		"SELECT day, COUNT(*) AS times FROM main WHERE year=$1 AND month=$2 GROUP BY day";
	let mut res = sqlx::query(SQL_SELECT).bind(year)
	                                     .bind(month)
	                                     .fetch(&mut **db);

	let mut ans: Vec<DayCount> = Vec::new();

	// TODO: 删除用于快速验证的错误转换
	while let Some(row) = res.try_next().await.map_err(|e| e.to_string())? {
		ans.push(DayCount{ day: row.get(0), times: row.get(1)});
	}

	Ok(Json(ans))
}

#[get("/fixed_dates?<offset>&<month>&<year>")]
fn fixed_dates(offset: i8, month: u8, year: i32) -> Result<String> {
	check_date(month, year)?;
	check_offset(offset)?;
	Ok(format!("{year}-{month:02} [{offset:+02}]"))
}


#[launch]
fn rocket() -> _ {
	rocket::build().attach(Records::init())
	               .mount("/stat", routes![dates, fixed_dates])
}
