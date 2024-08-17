use rocket::futures::TryStreamExt;
use rocket::serde::Serialize;
use rocket::time::{util, Date, Month, Time};
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

fn ordinals(offset: i8, month: u8, year: i32) -> Result<(i32, i32, u32)> {
	// 基准时间为 1970-01-01
	// 作为序号 0, 第一天
	let base_day = Date::from_ordinal_date(1970, 1).unwrap();

	let month = Month::try_from(month).map_err(|_| Error::DateWrong(year, month))?;
	let begin_day = Date::from_calendar_date(year, month, 1)?;
	// 查询起始日期的序号
	// 目前定义为本月第一天, 根据偏移量另作调整
	// 可能的值为: 本月第一天, 上个月最后一天
	let mut begin_day_ordinal = (begin_day - base_day).whole_days() as i32;
	// 本月的天数
	let days = util::days_in_year_month(year, month) as i32;

	// 当 offset < 0 时,
	// 范围是 上个月最后一天 到 本月最后一天.
	// 当 offset > 0 时
	// 范围是 本月第一天 到 下月第一天.
	// 当 offset == 0 时,
	// 范围是 本月第一天 到 本月最后一天.

	// 查询结束日期的序号
	// 可能的值为: 本月最后一天, 下月第一天
	let end_day_ordinal = if offset < 0 {
		begin_day_ordinal -= 1;
		begin_day_ordinal + days
	} else if offset > 0 {
		begin_day_ordinal + days
	} else {
		begin_day_ordinal + days - 1
	};

	Ok((begin_day_ordinal, end_day_ordinal, days as u32))
}

// 用于根据给定的偏移值计算补偿查询得到的日期序号的修正值
// 可能的返回值为: -1, 0, 1
// 分别代表: 前一天, 今天, 后一天
fn fix_ordinal_by_time(offset: i8, time: Time) -> i32 {
	let start = ((offset + 24) % 24) as f32;
	// 因为偏移量设计为仅支持整小时数, 所以只需精确到下一位 (分钟) 即可
	let time = time.hour() as f32 + time.minute() as f32 / 60.;

	return if start <= time && offset < 0 {
		1
	} else if start > time && offset > 0 {
		-1
	} else {
		0
	};
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

pub async fn select_dates(buf: &mut MonthCount, month: u8, year: i32,
                          mut db: Connection<Records>)
                          -> Result<()> {
	assert!(buf.is_empty());
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

pub async fn select_fixed_dates(buf: &mut MonthCount, offset: i8, month: u8, year: i32,
                                mut db: Connection<Records>)
                                -> Result<()> {
	assert!(buf.is_empty());
	check_offset(offset)?;
	let (begin, end, days) = ordinals(offset, month, year)?;
	let month_day_ordinal = begin + if offset < 0 { 1 } else { 0 };

	const SQL_SELECT: &str = "SELECT ordinal, time FROM main INNER JOIN extrainfo AS ex ON \
	                          main.id=ex.main_id WHERE ordinal BETWEEN $1 AND $2";
	let mut res = sqlx::query(SQL_SELECT).bind(begin)
	                                     .bind(end)
	                                     .fetch(&mut **db);

	for day in 1..=31 {
		buf.push(DayCount { day, times: 0 });
	}

	while let Some(row) = res.try_next().await? {
		let time: Option<Time> = row.get(1);
		let day = row.get::<i32, _>(0) - month_day_ordinal
		          + time.map_or(0, |t| fix_ordinal_by_time(offset, t))
		          + 1;

		if day < 1 || day > days as i32 {
			continue;
		}

		buf.get_mut(day as usize - 1)
		   .ok_or(Error::from("Unknown error"))?
		   .times += 1;
	}

	buf.retain(|dt| dt.times > 0);

	Ok(())
}
