//! Error variants

pub type DcsvResult<T> = Result<T, DcsvError>;

/// Error types for dcsv related operations.
#[derive(Debug)]
pub enum DcsvError {
    InvalidLimiter(String),
    InvalidValueType(String),
    IoError(IoErrorWithMeta),
    OutOfRangeError,
    InsufficientRowData,
    InvalidRowData(String),
    InvalidColumn(String),
    InvalidCellData(String),
    CommandError(String),
}

impl std::fmt::Display for DcsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimiter(txt) => write!(f, "ERR : Invalid limiter =\n{0}", txt),
            Self::InvalidValueType(txt) => write!(f, "ERR : Invalid type =\n{0}", txt),
            Self::IoError(io_error) => write!(f, "ERR : IO Error =\n{0}", io_error),
            Self::OutOfRangeError => write!(f, "ERR : Index out of range"),
            Self::InsufficientRowData => write!(f, "ERR : Insufficient row data"),
            Self::InvalidRowData(txt) => write!(f, "ERR : Invalid row data =\n{0}", txt),
            Self::InvalidColumn(txt) => write!(f, "ERR : Invalid column =\n{0}", txt),
            Self::InvalidCellData(txt) => write!(f, "ERR : Invalid cell data =\n{0}", txt),
            Self::CommandError(txt) => write!(f, "ERR : Invalid command call =\n{0}", txt),
        }
    }
}

impl DcsvError {
    pub fn io_error(err: std::io::Error, meta: &str) -> Self {
        Self::IoError(IoErrorWithMeta::new(err, meta))
    }
}

/// Specific error struct with meta information
pub struct IoErrorWithMeta {
    error: std::io::Error,
    meta: String,
}

impl IoErrorWithMeta {
    /// Create a new instance
    pub fn new(error: std::io::Error, meta: &str) -> Self {
        Self {
            error,
            meta: meta.to_owned(),
        }
    }
}

impl std::fmt::Debug for IoErrorWithMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} :: {}", self.error, self.meta)
    }
}

impl std::fmt::Display for IoErrorWithMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} :: {}", self.error, self.meta)
    }
}
