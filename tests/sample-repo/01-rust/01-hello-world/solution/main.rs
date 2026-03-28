/// Returns the classic "Hello, World!" greeting.
fn greeting() -> &'static str {
    "Hello, World!"
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
}