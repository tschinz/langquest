use sha2::{Digest, Sha256};

fn hex_hash(str: &str) -> String {
  Sha256::digest(str).map(|a| format!("{a:x}")).join("")
}

fn main() {
  println!("{}", hex_hash("awawa"));
}

//==============================================================================
//                           EXERCISE UNIT TESTS
//                       DO NOT EDIT BELOW THIS LINE
//==============================================================================
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_1() {
    assert_eq!(hex_hash("test1!"), "7c4a7b676e873b49d643151b7675e9b51040be4bd64bfdcd51743942ac5bb");
  }

  #[test]
  fn test_2() {
    assert_eq!(hex_hash("test2"), "60303ae22b998861bce3b28f33eec1be758a213c86c93c76dbe9f558c11c752");
  }

  #[test]
  fn test_empty() {
    assert_eq!(hex_hash(""), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
  }
}
