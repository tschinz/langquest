// TODO: implement the greeting function
fn greeting() -> &'static str {
  "Hello, World!"k
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
