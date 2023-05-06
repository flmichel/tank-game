use axum::http::StatusCode;

pub enum Result<T> {
    Ok(T),
    Err(Error),
}

impl<T> Result<T> {
    pub fn explain_error(self, explanation: impl Into<String>) -> Result<T> {
        match self {
            Result::<T>::Ok(_) => self,
            Result::Err(error) => Result::Err(error.explain(explanation)),
        }
    }

    pub fn wrap(result: std::result::Result<T, impl ToString>, kind: ErrorKind) -> Result<T> {
        match result {
            Ok(result) => Result::Ok(result),
            Err(error) => Result::Err(Error::from(error, kind)),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: Vec<String>,
}

impl Error {
    pub fn new(kind: ErrorKind, description: String) -> Error {
        Error {
            kind,
            description: vec![description],
        }
    }

    pub fn from(error: impl ToString, kind: ErrorKind) -> Error {
        Error {
            kind,
            description: vec![error.to_string()],
        }
    }

    pub fn explain(mut self, explanation: impl Into<String>) -> Error {
        self.description.push(explanation.into());
        self
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    ConfigurationError,
    ParsingError,
    NetworkError,
    StartupError,
    HttpError(StatusCode),
}
