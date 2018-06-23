extern crate curl;
use curl::easy::Easy;

use std::str;
use std::io::{stdout, Write};

pub fn fetch(url : &str) -> () {
    let mut easy = Easy::new();
    easy.url(url).unwrap();

    easy.header_function(|header| {
        print!("header: {}", str::from_utf8(header).unwrap());
        true
    }).unwrap();

    easy.write_function(|data| {
        Ok(stdout().write(data).unwrap())
    }).unwrap();

    easy.perform().unwrap();
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
