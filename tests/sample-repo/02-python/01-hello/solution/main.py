# Exercise 01: Hello, Python! — Solution


def greeting():
    """Return the string "Hello, World!" exactly."""
    return "Hello, World!"


def greet(name):
    """Return a personalised greeting: "Hello, {name}!" using an f-string."""
    return f"Hello, {name}!"


def my_abs(n):
    """Return the absolute value of n WITHOUT using the built-in abs()."""
    if n < 0:
        return -n
    return n


def is_leap_year(year):
    """
    Return True if year is a leap year according to the Gregorian calendar:
      - divisible by 4        → leap year
      - except divisible by 100 → NOT a leap year
      - except divisible by 400 → leap year
    """
    return (year % 4 == 0 and year % 100 != 0) or (year % 400 == 0)


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

import unittest


class TestGreeting(unittest.TestCase):
    def test_returns_hello_world(self):
        self.assertEqual(greeting(), "Hello, World!")

    def test_not_empty(self):
        self.assertIsNotNone(greeting())
        self.assertNotEqual(greeting(), "")

    def test_starts_with_hello(self):
        self.assertTrue(greeting().startswith("Hello"))

    def test_ends_with_exclamation(self):
        self.assertTrue(greeting().endswith("!"))

    def test_exact_type(self):
        self.assertIsInstance(greeting(), str)


class TestGreet(unittest.TestCase):
    def test_greet_alice(self):
        self.assertEqual(greet("Alice"), "Hello, Alice!")

    def test_greet_bob(self):
        self.assertEqual(greet("Bob"), "Hello, Bob!")

    def test_greet_empty_string(self):
        self.assertEqual(greet(""), "Hello, !")

    def test_greet_with_spaces(self):
        self.assertEqual(greet("Mary Jane"), "Hello, Mary Jane!")

    def test_greet_returns_string(self):
        self.assertIsInstance(greet("test"), str)


class TestMyAbs(unittest.TestCase):
    def test_positive_number(self):
        self.assertEqual(my_abs(5), 5)

    def test_negative_number(self):
        self.assertEqual(my_abs(-5), 5)

    def test_zero(self):
        self.assertEqual(my_abs(0), 0)

    def test_large_negative(self):
        self.assertEqual(my_abs(-1000000), 1000000)

    def test_negative_one(self):
        self.assertEqual(my_abs(-1), 1)

    def test_float_negative(self):
        self.assertAlmostEqual(my_abs(-3.14), 3.14)

    def test_float_positive(self):
        self.assertAlmostEqual(my_abs(2.71), 2.71)


class TestIsLeapYear(unittest.TestCase):
    def test_typical_leap_year(self):
        self.assertTrue(is_leap_year(2024))

    def test_typical_non_leap_year(self):
        self.assertFalse(is_leap_year(2023))

    def test_century_not_leap(self):
        self.assertFalse(is_leap_year(1900))

    def test_400_year_leap(self):
        self.assertTrue(is_leap_year(2000))

    def test_another_century_not_leap(self):
        self.assertFalse(is_leap_year(1800))

    def test_another_400_year_leap(self):
        self.assertTrue(is_leap_year(1600))

    def test_non_leap_odd(self):
        self.assertFalse(is_leap_year(2019))

    def test_leap_2020(self):
        self.assertTrue(is_leap_year(2020))

    def test_century_2100_not_leap(self):
        self.assertFalse(is_leap_year(2100))

    def test_year_4(self):
        self.assertTrue(is_leap_year(4))


if __name__ == "__main__":
    unittest.main()