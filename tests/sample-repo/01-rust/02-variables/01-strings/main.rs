// TODO: implement the concat function
fn concat(a: &str, b: &str) -> String {
  String::new()
}

fn main() {
    println!("'{}'", concat("Hello", "World"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_hello_world() {
        assert_eq!(concat("Hello", "World"), "Hello World");
    }

    #[test]
    fn test_concat_foo_bar() {
        assert_eq!(concat("foo", "bar"), "foo bar");
    }

    #[test]
    fn test_concat_empty_first() {
        assert_eq!(concat("", "test"), " test");
    }
}
