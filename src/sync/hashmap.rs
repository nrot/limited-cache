use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::iter::Iterator;
use std::ops::Deref;
use std::path::PathBuf;

use crate::error::Error;

#[derive(Debug)]
pub struct HashMapCache<K: Sized, V: Sized, S, D>
where
    S: Fn(&V) -> Result<Vec<u8>, Error>,
    D: Fn(Vec<u8>) -> Result<HashMap<K, V>, Error>,
{
    file: File,
    data: HashMap<K, V>,
    limit: usize,
    serializer: S,
    deserializer: D,
}

