use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use std::iter::Iterator;
use std::path::PathBuf;

use crate::error::Error;

#[derive(Debug)]
pub struct AsyncVecCache<T: Sized, S, D>
where
    S: Fn(&T) -> Result<Vec<u8>, Error>,
    D: Fn(Vec<u8>) -> Result<Vec<T>, Error>,
{
    file: File,
    data: Vec<T>,
    limit: usize,
    serializer: S,
    deserializer: D,
}

impl<T, S, D> AsyncVecCache<T, S, D>
where
    T: Sized,
    S: Fn(&T) -> Result<Vec<u8>, Error>,
    D: Fn(Vec<u8>) -> Result<Vec<T>, Error>,
{
    pub async fn create(
        path: PathBuf,
        limit: Option<usize>,
        serializer: S,
        deserializer: D,
    ) -> Result<AsyncVecCache<T, S, D>, Error> {
        let file = File::create(path).await?;
        Ok(AsyncVecCache {
            file,
            data: Vec::new(),
            limit: limit.unwrap_or(usize::MAX),
            serializer,
            deserializer,
        })
    }

    pub async fn open(
        path: PathBuf,
        limit: Option<usize>,
        serializer: S,
        deserializer: D,
    ) -> Result<AsyncVecCache<T, S, D>, Error> {
        let file = File::open(path).await?;
        let mut s = AsyncVecCache {
            file,
            data: Vec::new(),
            limit: limit.unwrap_or(usize::MAX),
            serializer,
            deserializer,
        };
        let mut buff = Vec::new();
        let _ = s.file.read_to_end(&mut buff).await?;
        let data = (s.deserializer)(Vec::from(buff))?;
        s.data = data;
        Ok(s)
    }

    pub fn sync_limit(&mut self) -> &Self {
        while self.data.len() > self.limit {
            self.data.pop();
        }
        self
    }

    pub async fn flush(&mut self) -> Result<usize, Error> {
        self.file.set_len(0).await?;
        self.file.sync_all().await?;
        self.file.rewind().await?;

        let i = if self.limit > self.data.len() {
            0 as usize
        } else {
            self.data.len() - self.limit
        };
        let iter = self.data.iter().skip(i);
        for d in iter {
            let data = (self.serializer)(d)?;
            self.file.write_all(&data).await?;
        }
        Ok(self.limit)
    }

    pub async fn sync_all(&mut self) -> Result<(), Error> {
        match self.file.sync_all().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn push(&mut self, data: T) -> Result<&Self, Error> {
        let to_write = (self.serializer)(&data)?;
        self.data.push(data);
        if self.data.len() < self.limit {
            return match self.file.write_all(&to_write).await {
                Ok(_) => Ok(self),
                Err(e) => Err(e.into()),
            };
        };
        self.sync_limit();
        match self.flush().await {
            Ok(_) => Ok(self),
            Err(e) => Err(e),
        }
    }

    pub fn push_buf(&mut self, data: T) -> &Self {
        self.data.push(data);
        self.sync_limit();
        self
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn iter(&self)->std::slice::Iter<'_, T>{
        self.data.iter()
    }
}
