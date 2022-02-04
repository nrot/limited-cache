use std::fs::File;
use std::io::{Read, Seek, Write};
use std::iter::Iterator;
use std::path::PathBuf;

use crate::error::Error;

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
    ) -> Result<VecCache<T, S, D>, Error> {
        let file = File::create(path)?;
        Ok(VecCache {
            file,
            data: Vec::new(),
            limit: limit.unwrap_or(usize::MAX),
            serializer,
            deserializer,
        })
    }

    pub fn open(path: PathBuf, limit: Option<usize>, serializer: S,
                deserializer: D)->Result<VecCache<T, S, D>, Error>{
        let file = File::open(path)?;
        let mut s = VecCache{
           file,
            data: Vec::new(),
            limit: limit.unwrap_or(usize::MAX),
            serializer,
            deserializer
        };
        let mut buff = Vec::new();
        let _ = s.file.read_to_end(&mut buff)?;
        let data = (s.deserializer)(Vec::from(buff))?;
        s.data = data;
        Ok(s)
    }

    pub fn sync_limit(&mut self)->&Self{
        while self.data.len() > self.limit{
            self.data.pop();
        };
        self
    }

    pub fn flush(&mut self) -> Result<usize, Error> {
        self.file.set_len(0)?;
        self.file.sync_all()?;
        self.file.rewind()?;

        let i = if self.limit > self.data.len(){
            0 as usize
        } else {
            self.data.len() - self.limit
        };
        let mut iter = self.data.iter().skip(i);
        while let Some(d) = iter.next() {
            let data = (self.serializer)(d)?;
            self.file.write_all(&data)?;
        }
        Ok(self.limit)
    }

    pub fn sync_all(&mut self)->Result<(), Error>{
        match self.file.sync_all(){
            Ok(_)=>Ok(()),
            Err(e)=>Err(e.into())
        }
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
        self.sync_limit();
        match self.flush() {
            Ok(_) => { Ok(self) }
            Err(e) => { Err(e) }
        }
    }

    pub fn push_buf(&mut self, data: T)->&Self{
        self.data.push(data);
        self.sync_limit();
        self
    }

    pub fn len(&self)->usize{
        self.data.len()
    }
}
