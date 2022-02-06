mod error;
mod sync;
mod tsync;

pub use sync::VecCache;

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::VecCache;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn lib() {
        use serde::{self, Serialize, Deserialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct Data {
            pub text: String,
        }

        use std::str;

        fn s(d: &Data) -> Result<Vec<u8>, Error> {
            return match serde_json::to_string(d) {
                Ok(d) => Ok(Vec::from(d.as_bytes())),
                Err(_) => { Err(Error::serialize()) }
            };
        }
        fn d(d: Vec<u8>) -> Result<Vec<Data>, Error> {
            let data = match str::from_utf8(&d) {
                Ok(d) => d,
                Err(e) => return Err(e.into())
            };
            return match serde_json::from_str(&data) {
                Ok(d) => Ok(d),
                Err(_) => Err(Error::deserialize())
            };
        }

        use std::path::Path;
        let p = Path::new("./tmp/some_guid");
        let mut c = VecCache::create(p.to_path_buf(), None, s, d).unwrap();
        assert_eq!(1, c.push(Data {
            text: "test".into()
        }).unwrap().len());
    }
}
