
use nanoid::nanoid;


#[derive(Debug, Clone, PartialEq)]
pub struct TinyUrl {
    original_url: String,
    size: usize,
}

impl TinyUrl {
    pub fn new(url: &str, size: usize) -> Self {
        let cleaned_url = url.to_owned()
        .chars()
        .filter(|char| char.is_alphanumeric()).collect::<String>();

        Self { original_url: cleaned_url, size }
    }

    pub fn get_tiny_url(&self) -> String {
        let chars = self.original_url.chars().collect::<Vec<char>>();

        let tiny = nanoid!(self.size, &chars);

        println!("{}",tiny.as_str());

        tiny
    }
}

#[cfg(test)]
mod test {
    use sqlx::test;

use super::*;

    fn get_tiny_url(url: &str, mut size: usize) -> String {
        if size.lt(&0usize) {
            size = 1usize;
        }
        let init = TinyUrl::new(url, size);

        init.get_tiny_url()
    }

    #[test]
    fn test_single_char() {
        assert_eq!(1, get_tiny_url("http://google.com",1).len(), "testing that get tiny url returns empty string when empty")
    }

    #[test]
    fn test_10_char() {
        assert_eq!(10, get_tiny_url("http://google.com",10).len(), "testing that get tiny url returns empty string when empty")
    }
}

