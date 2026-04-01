"""Solution for Hello, Python! exercise."""

def greeting():
    """Return the classic greeting string.

    Returns:
        str: The greeting "Hello, World!" exactly as specified.
    """
    return "Hello, World!"

# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------
import unittest

class TestGreeting(unittest.TestCase):
    def test_returns_hello_world(self):
        self.assertEqual(greeting(), "Hello, World!")

if __name__ == "__main__":
    unittest.main()
