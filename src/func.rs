use rocket::futures::TryStreamExt;
use rocket::serde::Serialize;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::{Connection, Database};

use crate::error::*;


#[derive(Database)]
#[database("records")]
pub struct Records(sqlx::SqlitePool);


/// 检查日期是否正确
///
/// - 检查月份是否存在: [1, 12]
fn check_date(month: u8, year: i32) -> Result<()> {
	if month < 1 || month > 12 {
		return Err(Error::DateWrong(year, month));
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
		return Err(Error::OffsetWrong(offset));
	}

	Ok(())
}

/// 某日的次数统计
#[derive(Serialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(crate = "rocket::serde")]
pub struct DayCount {
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

pub type MonthCount = Vec<DayCount>;

pub async fn select_dates(buf: &mut Vec<DayCount>, month: u8, year: i32,
                          mut db: Connection<Records>)
                          -> Result<()> {
	check_date(month, year)?;

	const SQL_SELECT: &str =
		"SELECT day, COUNT(*) AS times FROM main WHERE year=$1 AND month=$2 GROUP BY day";
	let mut res = sqlx::query(SQL_SELECT).bind(year)
	                                     .bind(month)
	                                     .fetch(&mut **db);

	while let Some(row) = res.try_next().await? {
		buf.push(DayCount { day:   row.get(0),
		                    times: row.get(1), });
	}

	Ok(())
}
