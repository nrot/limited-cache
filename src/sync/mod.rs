use std::borrow::Borrow;
use std::error;
use std::fs::File;
use std::io::{self, BufWriter, Seek, Write};
use std::iter::Iterator;
use std::path::PathBuf;
use std::iter::DoubleEndedIterator;

pub struct Error;

#[derive(Debug)]
pub struct VecCache<T: Sized, S, D>
    where S: Fn(&T) -> Result<Vec<u8>, Error>,
          D: Fn(Vec<u8>) -> Result<Vec<T>, Error>
{
    file: File,
    data: Vec<T>,
    limit: usize,
    vec_limit: bool,
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
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        Ok(VecCache {
            file,
            data: Vec::new(),
            limit: match limit { Some(l)=>l, None=>usize::MAX },
            vec_limit: true,
            serializer,
            deserializer,
        })
    }

    pub fn flush(&mut self) -> io::Result<usize> {
        match self.file.set_len(0) {
            Ok(_) => {}
            Err(e) => return Err(e)
        };
        match self.file.rewind(){
            Ok(_)=>{},
            Err(e)=>{return Err(e)}
        };
        let mut iter = self.data.iter();
        let mut i = 0;
        while let Some(d) = iter.next_back() {
            if i >= self.limit{
                break;
            }
            i += 1;
            let data = match (self.serializer)(d) {
                Ok(d) => d,
                Err(_) => return Err(io::Error::last_os_error())
            };
            match self.file.write_all(&data) {
                Ok(_) => {}
                Err(e) => return Err(e)
            };
        }
        Ok(self.limit)
    }
}
