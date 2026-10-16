use std::fmt;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: Vec<String>,
}

impl Error {
    pub fn new<D>(kind: ErrorKind, description: D) -> Error
    where
        D: Into<String>,
    {
        Error {
            kind,
            description: vec![description.into()],
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.description.join(": "))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    ConfigurationError,
    NoSpawnAvailable,
}
