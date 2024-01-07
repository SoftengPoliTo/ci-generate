use thiserror::Error;

///Error types
#[derive(Debug, Error)]
pub enum Error {
    /// Wrong expansion to home directory.
    #[error("Wrong expansion")]
    WrongExpandUser,
    /// Impossible to canonicalize this path.
    #[error("Canonicalization failure")]
    CanonicalPath,
    /// Impossible to create this path.
    #[error("Path creation failure")]
    CreationError,
    /// Unable to retrieve the home directory.
    #[error("Home directory failure")]
    HomeDir,
    /// A general utf-8 conversion error.
    #[error("Utf-8 error")]
    UTF8Check,
    /// License not found.
    #[error("License not found")]
    NoLicense,
    /// Invalid license
    #[error("Invalid license")]
    InvalidLicense,
    /// A more generic I/O error.
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    /// A minijinja error.
    #[error("Minijinja error")]
    Minijinja(#[from] minijinja::Error),
}

/// A specialized `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;
