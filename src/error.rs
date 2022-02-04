
#[derive(Debug)]
pub struct Error{
    error: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind{
    Io(std::io::Error),
    Serialize(),
    Deserialize(),
    Utf8(std::str::Utf8Error)
}

impl Error {
    pub fn serialize()->Self{
        Error{
            error:ErrorKind::Serialize()
        }
    }
    pub fn deserialize()->Self{
        Error{
            error:ErrorKind::Deserialize()
        }
    }
}


impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error{
            error: ErrorKind::Io(e),
        }
    }
}

impl From<std::str::Utf8Error> for Error{
    fn from(e: std::str::Utf8Error)->Self{
        Error{
            error: ErrorKind::Utf8(e)
        }
    }
}