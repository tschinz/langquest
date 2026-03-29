# TODO: implement the greeting function
def greeting():
    return "Hello, World!"


import unittest

class TestGreeting(unittest.TestCase):
    def test_greeting(self):
        self.assertEqual(greeting(), "Hello, World!")

if __name__ == "__main__":
    unittest.main()
