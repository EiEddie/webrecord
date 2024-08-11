use rocket::http::Status;
use thiserror::Error;


#[derive(Debug, Error)]
pub enum Error {
	#[error("{0}")]
	Msg(String),

	/// 日期错误
	#[error("'{0}-{1:02}' is not an available year and month")]
	DateWrong(i32, u8),

	/// 偏移量不合法
	#[error("Offset [{0:+2}] is not allowed")]
	OffsetWrong(i8),

	/// 数据库错误
	///
	/// 由 `sqlx` 产生
	#[error(transparent)]
	DatabaseErr(#[from] ::rocket_db_pools::sqlx::Error),

	/// 日期与时间转换错误
	///
	/// 由 `time` 产生
	///
	/// 与 [`Error::DateWrong`] 不同, 此为服务器内部出现的错误,
	/// 并非用户输入问题. 不应暴露给用户.
	#[error(transparent)]
	DateTimeErr(#[from] ::rocket::time::error::ComponentRange),
}

impl From<&'static str> for Error {
	fn from(s: &'static str) -> Self {
		Error::Msg(s.to_owned())
	}
}

impl From<String> for Error {
	fn from(s: String) -> Self {
		Error::Msg(s)
	}
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub fn err_status(res: Result<()>) -> Status {
	let err = res.err();
	if let Some(err) = err {
		match err {
			Error::DateWrong(..) | Error::OffsetWrong(_) => Status::BadRequest,
			Error::DatabaseErr(_) | Error::DateTimeErr(_) => Status::InternalServerError,
			Error::Msg(_) => Status::InternalServerError,
		}
	} else {
		Status::Ok
	}
}
