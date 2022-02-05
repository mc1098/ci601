pub(crate) type DynError = Box<dyn std::error::Error + Send + Sync>;

/// The Errors that may occur when calling the seb functions.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    message: Option<String>,
    source: Option<DynError>,
}

/// Types of errors that make up an [`Error`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorKind {
    /// The error is associated with an underlying IO error.
    IO,
    /// An error caused when parsing/deserialization fails.
    Deserialize,
    /// An error when an operation has failed to return a value.
    NoValue,
}

impl Error {
    /// Creates a new [`Error`] based on the [`ErrorKind`] and message to describe the error.
    pub fn new<S: Into<String>>(kind: ErrorKind, message: S) -> Self {
        Self {
            kind,
            message: Some(message.into()),
            source: None,
        }
    }

    /// Wraps an existing error as the source of [`Error`].
    pub fn wrap<E>(kind: ErrorKind, source: E) -> Self
    where
        E: Into<DynError>,
    {
        Self {
            kind,
            message: None,
            source: Some(source.into()),
        }
    }

    /// Returns the kind of error.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::IO => f.write_str("IO error")?,
            ErrorKind::Deserialize => f.write_str("Deserialize error")?,
            ErrorKind::NoValue => f.write_str("No value error")?,
        };

        if let Some(message) = &self.message {
            write!(f, ": {message}")?;
        }

        if let Some(cause) = &self.source {
            write!(f, ": caused by {cause}")?;
        }
        Ok(())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}
