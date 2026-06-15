// TODO: implement the boolean functions
fn is_even(n: i32) -> bool {
  false
}

fn is_positive(n: i32) -> bool {
  false
}

fn both_even(a: i32, b: i32) -> bool {
  false
}

fn main() {
    println!("is_even(4) = {}", is_even(4));
    println!("is_positive(-3) = {}", is_positive(-3));
    println!("both_even(2, 8) = {}", both_even(2, 8));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_even_true() {
        assert!(is_even(4));
    }

    #[test]
    fn test_is_even_false() {
        assert!(!is_even(7));
    }

    #[test]
    fn test_is_positive_true() {
        assert!(is_positive(5));
    }

    #[test]
    fn test_is_positive_false() {
        assert!(!is_positive(-3));
    }

    #[test]
    fn test_both_even_true() {
        assert!(both_even(2, 8));
    }

    #[test]
    fn test_both_even_false() {
        assert!(!both_even(3, 6));
    }
}
