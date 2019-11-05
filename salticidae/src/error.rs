use std::io;

#[derive(Debug)]
pub enum Error {
    InvalidChecksum,
    StreamClosed,
    IOError { message: String },
}
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IOError {
            message: error.to_string(),
        }
    }
}
