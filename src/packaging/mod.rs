use std::fmt::Display;

pub mod jar;

// ========================================================================
// ERRORS
// ========================================================================

#[derive(Debug)]
pub enum ClassReadingError {
    // IoError(std::io::Error),
    ZipError(zip::result::ZipError),
}

impl Display for ClassReadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // ClassReadingError::IoError(err) => writeln!(f, "{}", err),
            ClassReadingError::ZipError(err) => writeln!(f, "{}", err),
        }
    }
}
