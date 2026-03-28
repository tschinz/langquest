// Exercise 01: Hello World
//
// Your first Rust program! In this exercise you will implement a simple
// function that returns a greeting string.
//
// TODO: Implement the `greeting` function so that it returns the string
//       "Hello, World!" exactly (including punctuation and capitalization).

// TODO: implement the greeting function
/// Returns the classic "Hello, World!" greeting.
fn greeting() -> &'static str {
    "Hello, Worfld!"
}
fn main() {
    println!("{}", greeting());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting() {
        assert_eq!(greeting(), "Hello, World!");
    }

    #[test]
    fn test_greeting_not_empty() {
        assert!(!greeting().is_empty(), "greeting() should not return an empty string");
    }

    #[test]
    fn test_greeting_starts_with_hello() {
        assert!(
            greeting().starts_with("Hello"),
            "greeting() should start with 'Hello'"
        );
    }
}
