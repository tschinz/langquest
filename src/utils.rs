//! This module contains various utils, abstracted for use in other modules.

use std::io::{self, Write};

/// Builder for yes/no questions.
///
/// ```no_run
/// use lq::utils::Question;
///
/// let answer = Question::ask("Continue?").default(true).prompt().unwrap();
/// if answer {
///   println!("Question answered with yes!");
/// } else {
///   println!("Question answered with no!");
/// }
/// ```
pub struct Question {
  msg: String,
  default: Option<bool>,
  exact_yes: Option<String>,
}

impl Question {
  /// Create a yes/no question with the given prompt text.
  pub fn ask(msg: impl Into<String>) -> Self {
    Self {
      msg: msg.into(),
      default: None,
      exact_yes: None,
    }
  }

  /// Set the exact required answer to count as 'yes'.
  /// The answer set here is still case-insensitive.
  /// Mutually exclusive with `default(true)`.
  pub fn exact_yes(mut self, exact_yes: impl Into<String>) -> Self {
    self.exact_yes = Some(exact_yes.into().to_ascii_lowercase());
    self
  }

  /// Set the default answer, used when the user presses Enter.
  /// It is shown in the prompt as [y/N] or [Y/n].
  /// `default(true)` is mutually exclusive with `exact_yes`.
  pub fn default(mut self, default: bool) -> Self {
    self.default = Some(default);
    self
  }

  /// Print the prompt and read an answer from stdin.
  /// Accepts y/yes and n/no by default, enter accepts the default.
  pub fn prompt(self) -> io::Result<bool> {
    assert!(self.default != Some(true) || self.exact_yes.is_none());
    let exact_yes = self.exact_yes.unwrap_or_default();
    let yes_answers = if !exact_yes.is_empty() { vec![exact_yes.as_str()] } else { vec!["y", "yes"] };

    let hint: Option<String> = match self.default {
      Some(true) => Some("Y/n".into()),
      Some(false) => Some("y/N".into()),
      None => {
        if exact_yes.is_empty() {
          Some("y/n".into())
        } else {
          Some(format!("{}/n", exact_yes))
        }
      }
    };

    let prompt = hint.map_or(self.msg.clone(), |h| format!("{} [{}]", self.msg, h));

    loop {
      print!("{prompt}: ");
      io::stdout().flush()?;

      let mut input = String::new();
      io::stdin().read_line(&mut input)?;
      let input = input.trim().to_ascii_lowercase();

      match input.as_str() {
        "" => {
          if let Some(default) = self.default {
            return Ok(default);
          }
        }
        val if yes_answers.contains(&val) => return Ok(true),
        "n" | "no" => return Ok(false),
        _ => {}
      }
      println!("Invalid answer. Try again..");
    }
  }
}

/// Builder for picking one of a fixed set of options.
///
/// ```no_run
/// use lq::utils::Select;
///
/// let options = ["easy", "medium", "hard"];
/// let choice = Select::of("Choose an option: ", &options).default(1).prompt().unwrap();
/// ```
pub struct Select<'a> {
  msg: String,
  options: &'a [&'a str],
  default: Option<usize>,
}

impl<'a> Select<'a> {
  /// Create a selection prompt with the given options.
  /// Panics when `options` is empty.
  pub fn of(msg: impl Into<String>, options: &'a [&'a str]) -> Self {
    assert!(!options.is_empty(), "Select needs at least one option");
    Self {
      msg: msg.into(),
      options,
      default: None,
    }
  }

  /// Set the default option by index. It is marked with `>` in the list and
  /// shown as the prompt hint.
  pub fn default(mut self, default: usize) -> Self {
    assert!(default < self.options.len(), "default index out of bounds");
    self.default = Some(default);
    self
  }

  /// Print the numbered options and read a choice from stdin.
  /// Returns the index of the chosen option. Enter selects the default of set.
  pub fn prompt(self) -> io::Result<usize> {
    loop {
      println!("{}", self.msg);
      for (i, option) in self.options.iter().enumerate() {
        let marker = if self.default == Some(i) { ">" } else { " " };
        println!("  {marker} {i}. {option}");
      }
      print!("Enter a number: ");
      io::stdout().flush()?;

      let mut input = String::new();
      io::stdin().read_line(&mut input)?;
      let input = input.trim();

      if input.is_empty() {
        if let Some(i) = self.default {
          return Ok(i);
        }
      } else if let Ok(n) = input.parse::<usize>()
        && n < self.options.len()
      {
        return Ok(n);
      }
      println!("Invalid answer. Try again..");
    }
  }
}
