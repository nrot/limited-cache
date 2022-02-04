mod sync;

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
    #[test]
    fn lib() {
        use serde_json::{from_str, to_string};
        use std::path::Path;
        let p = Path::new("./tmp");
        let c = sync::Cache::new(p.to_path_buf(), None, to_string, from_str);
    }
}
