/// Returns the greeting string "Hello, World!"
/// 
/// In Rust, string literals are of type `&'static str` - a reference
/// to a string that lives for the entire program duration.
fn greeting() -> &'static str {
    // No semicolon = this expression is returned
    "Hello, World!"
}

fn main() {
    // Print the greeting to stdout
    println!("{}", greeting());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greeting() {
        assert_eq!(greeting(), "Hello, World!");
    }
}