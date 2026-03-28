---
id          = "hello_python"
name        = "Hello, Python!"
language    = "python"
difficulty  = 1
description = "Write your first Python functions — greetings, absolute value, and leap year detection."
topics      = ["functions", "strings", "f_strings", "basics"]
---

# Hello, Python!

## Objective

Implement four small functions that exercise the fundamentals of Python:
string returns, f-string formatting, conditional logic, and arithmetic.

## Instructions

Open `main.py` and complete the following functions:

### 1. `greeting()`

Return the string `"Hello, World!"` — exactly, including capitalisation and
punctuation.

### 2. `greet(name)`

Return a personalised greeting using an **f-string**:

```
greet("Alice")  → "Hello, Alice!"
greet("Bob")    → "Hello, Bob!"
```

### 3. `my_abs(n)`

Return the absolute value of `n` **without** using the built-in `abs()`
function. Use conditional logic instead.

```
my_abs(-5)  → 5
my_abs(3)   → 3
my_abs(0)   → 0
```

### 4. `is_leap_year(year)`

Return `True` if `year` is a leap year, `False` otherwise. Remember the
Gregorian rules:

- Divisible by **4** → leap year,
- *except* divisible by **100** → not a leap year,
- *except* divisible by **400** → leap year.

```
is_leap_year(2024) → True
is_leap_year(1900) → False
is_leap_year(2000) → True
is_leap_year(2023) → False
```

## Requirements

- Do **not** use the built-in `abs()` in `my_abs`.
- Do **not** use `calendar.isleap` or any imports in `is_leap_year`.
- Do **not** modify the tests at the bottom of `main.py`.

## Running

```sh
# Run the tests
python main.py

# Or with verbose output
python -m unittest main -v
```

## Expected Test Results

All tests should pass with no errors or failures.