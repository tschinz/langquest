---
title    = "Hello, Python!"
hints    = [
    "Just return the string literal directly.",
    "Use the def keyword to define a function.",
    "The return statement sends a value back to the caller.",
]
keywords = []
---

## Explanation

Python functions are defined with `def` and use `return` to send values back.

```python
# Define a function that returns a greeting string
def greeting():
    return "Hello, World!"  # Return the exact string

# Tests verify the function works correctly
import unittest

class TestGreeting(unittest.TestCase):
    def test_returns_hello_world(self):
        # assertEqual checks if two values match
        self.assertEqual(greeting(), "Hello, World!")

if __name__ == "__main__":
    unittest.main()
```

**Key concepts:**
- `def` - defines a function
- `return` - sends a value back to the caller
- `unittest` - Python's built-in testing framework
- `assertEqual` - checks if expected matches actual
