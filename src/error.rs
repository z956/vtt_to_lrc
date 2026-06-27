#[derive(Debug)]
pub enum ProcessError {
    Io(std::io::Error),
    Parse(String),
}

impl From<std::io::Error> for ProcessError {
    fn from(e: std::io::Error) -> Self {
        ProcessError::Io(e)
    }
}
