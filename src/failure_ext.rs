use std::fmt::Display;
use std::path::Path;

use failure::{Error, Fail};

fn wrap_error<E, D>(err: E, msg: &str, data: D) -> Error
where
    E: Fail,
    D: Display,
{
    Error::from(err.context(format_err!("{} ({})", msg, data)))
}

pub trait FmtResultExt<T, D> {
    fn context_fmt(self, msg: &str, data: D) -> Result<T, Error>;
}

impl<T, D: Display, E: Fail> FmtResultExt<T, D> for Result<T, E> {
    fn context_fmt(self, msg: &str, data: D) -> Result<T, Error> {
        self.map_err(|err| wrap_error(err, msg, data))
    }
}

pub trait PathResultExt<T> {
    fn context_path(self, msg: &str, path: &Path) -> Result<T, Error>;
}

impl<T, E: Fail> PathResultExt<T> for Result<T, E> {
    fn context_path(self, msg: &str, path: &Path) -> Result<T, Error> {
        self.map_err(|err| wrap_error(err, msg, path.display()))
    }
}

pub trait OptionFailExt<T> {
    fn or_fail(self, msg: &str) -> Result<T, Error>;
}

impl<T> OptionFailExt<T> for Option<T> {
    fn or_fail(self, msg: &str) -> Result<T, Error> {
        self.ok_or_else(|| format_err!("{}", msg))
    }
}
