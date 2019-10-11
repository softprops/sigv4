use rusoto_core::{credential::CredentialsError, request::HttpDispatchError};
use std::{error::Error as StdError, fmt};

#[derive(Debug)]
pub enum Error {
    Dispatch(HttpDispatchError),
    Credentials(CredentialsError),
}

impl fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> std::result::Result<(), fmt::Error> {
        match self {
            Error::Dispatch(ref err) => writeln!(f, "{}", err),
            Error::Credentials(ref err) => writeln!(f, "{}", err),
        }
    }
}

impl StdError for Error {}

impl From<HttpDispatchError> for Error {
    fn from(err: HttpDispatchError) -> Self {
        Error::Dispatch(err)
    }
}

impl From<CredentialsError> for Error {
    fn from(err: CredentialsError) -> Self {
        Error::Credentials(err)
    }
}
