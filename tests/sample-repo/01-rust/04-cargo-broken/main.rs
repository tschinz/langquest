/// The programming language of this course.
fn course_language() -> &'static str {
    "Rust"
}

/// The Rust edition this project is compiled with.
fn course_edition() -> &'static str {
    "2024"
}

fn main() {
    println!("This course teaches {} (edition {}).", course_language(), course_edition());
}

//==============================================================================
//                           EXERCISE UNIT TESTS
//                       DO NOT EDIT BELOW THIS LINE
//==============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language() {
        assert_eq!(course_language(), "Rust");
    }

    #[test]
    fn test_edition() {
        assert_eq!(course_edition(), "2024");
    }
}
