use std::borrow::Borrow;
use std::error;
use std::fs::File;
use std::io::{self, BufWriter, Seek, Write};
use std::iter::Iterator;
use std::path::PathBuf;
use std::iter::DoubleEndedIterator;
use std::str;


#[derive(Debug)]
pub struct Error{
    error: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind{
    Io(io::Error),
    Serialize(),
    Deserialize(),
    Utf8(str::Utf8Error)
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
    fn from(e: io::Error) -> Self {
        Error{
            error: ErrorKind::Io(e),
        }
    }
}

impl From<str::Utf8Error> for Error{
    fn from(e: str::Utf8Error)->Self{
        Error{
            error: ErrorKind::Utf8(e)
        }
    }
}

#[derive(Debug)]
pub struct VecCache<T: Sized, S, D>
    where S: Fn(&T) -> Result<Vec<u8>, Error>,
          D: Fn(Vec<u8>) -> Result<Vec<T>, Error>
{
    file: File,
    data: Vec<T>,
    limit: usize,
    serializer: S,
    deserializer: D,
}

impl<T, S, D> VecCache<T, S, D>
    where T: Sized,
          S: Fn(&T) -> Result<Vec<u8>, Error>,
          D: Fn(Vec<u8>) -> Result<Vec<T>, Error>
{
    pub fn create(
        path: PathBuf,
        limit: Option<usize>,
        serializer: S,
        deserializer: D,
    ) -> Result<VecCache<T, S, D>, io::Error> {
        let file = match File::create(path) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        Ok(VecCache {
            file,
            data: Vec::new(),
            limit: match limit {
                Some(l) => l,
                None => usize::MAX
            },
            serializer,
            deserializer,
        })
    }

    pub fn flush(&mut self) -> Result<usize, Error> {
        self.file.set_len(0)?;
        self.file.rewind()?;
        let mut iter = self.data.iter();
        let mut i = 0;
        while let Some(d) = iter.next_back() {
            if i >= self.limit {
                break;
            }
            i += 1;
            let data = (self.serializer)(d)?;
            self.file.write_all(&data)?;
        }
        Ok(self.limit)
    }

    pub fn push(&mut self, data: T) -> Result<&Self, Error> {
        let to_write = (self.serializer)(&data)?;
        self.data.push(data);
        if self.data.len() < self.limit {
            return match self.file.write_all(&to_write) {
                Ok(_) => Ok(self),
                Err(e) => Err(e.into())
            };
        };
        while self.data.len() > self.limit {
            self.data.pop();
        };
        match self.flush() {
            Ok(_) => { Ok(self) }
            Err(e) => { Err(e) }
        }
    }

    pub fn push_buf(&mut self, data: T)->&Self{
        self.data.push(data);
        while self.data.len() > self.limit{
            self.data.pop();
        };
        self
    }

    pub fn len(&self)->usize{
        self.data.len()
    }
}
