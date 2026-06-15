fn add(a: i32, b: i32) -> i32 {
  a + b
}

fn multiply(a: i32, b: i32) -> i32 {
  a * b
}

fn average(a: i32, b: i32) -> f64 {
  (a + b) as f64 / 2.0
}

fn main() {
    println!("add(3, 4) = {}", add(3, 4));
    println!("multiply(5, 6) = {}", multiply(5, 6));
    println!("average(10, 20) = {}", average(10, 20));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(3, 4), 7);
    }

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(5, 6), 30);
    }

    #[test]
    fn test_average() {
        assert!((average(10, 20) - 15.0).abs() < f64::EPSILON);
    }
}
