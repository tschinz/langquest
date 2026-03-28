# Hello, Python!

Every programmer's journey in a new language begins with a simple greeting.
In Python, this tradition is delightfully straightforward — but even the
simplest programs introduce important concepts worth understanding deeply.

## Functions

In Python, you define a function using the `def` keyword:

```python
def greeting():
    return "Hello, World!"
```

Key observations:

- `def` introduces a function definition.
- The function name is followed by **parentheses** `()` that hold any parameters.
- A **colon** `:` ends the function header.
- The function **body** is indented (typically 4 spaces).
- `return` sends a value back to the caller.

## Parameters and Arguments

Functions can accept **parameters** — placeholders for values you pass in:

```python
def greet(name):
    return f"Hello, {name}!"

print(greet("Alice"))  # Output: Hello, Alice!
```

- `name` is a **parameter** (in the definition).
- `"Alice"` is an **argument** (in the call).

## Strings and f-strings

Python has several ways to build strings:

### Concatenation

```python
"Hello, " + name + "!"
```

This works but gets unwieldy with many parts.

### The `format` Method

```python
"Hello, {}!".format(name)
```

Cleaner, but still a bit verbose.

### f-strings (Formatted String Literals)

```python
f"Hello, {name}!"
```

Introduced in Python 3.6, **f-strings** are the modern, preferred approach.
Prefix the string with `f` and embed expressions directly inside `{}`:

```python
x = 10
print(f"The square of {x} is {x ** 2}")
# Output: The square of 10 is 100
```

You can put **any valid Python expression** inside the braces — function
calls, arithmetic, method calls, and more.

## Conditional Logic

Python uses `if`, `elif`, and `else` to branch:

```python
def classify(n):
    if n > 0:
        return "positive"
    elif n < 0:
        return "negative"
    else:
        return "zero"
```

There is no need for parentheses around the condition (though they are
allowed). The colon and indentation define the block structure.

## Writing Your Own `abs` Without Built-ins

Python provides a built-in `abs()` function, but implementing it yourself
is a great exercise in conditional logic:

```python
def my_abs(n):
    if n < 0:
        return -n
    return n
```

This teaches you to think about **edge cases**: what happens when `n` is
zero? When it is already positive? The logic is simple, but reasoning
through every branch is a fundamental skill.

## Leap Year Rules

The Gregorian calendar defines a leap year with a specific set of rules:

1. A year **divisible by 4** is a leap year…
2. **except** years divisible by 100, which are **not** leap years…
3. **except** years divisible by 400, which **are** leap years.

Examples:

| Year | Divisible by 4? | Divisible by 100? | Divisible by 400? | Leap? |
|------|-----------------|--------------------|--------------------|-------|
| 2024 | Yes             | No                 | No                 | Yes   |
| 1900 | Yes             | Yes                | No                 | No    |
| 2000 | Yes             | Yes                | Yes                | Yes   |
| 2023 | No              | No                 | No                 | No    |

In Python, the **modulo operator** `%` checks divisibility:

```python
if year % 4 == 0:
    # divisible by 4
```

Combining conditions with `and`, `or`, and `not` lets you express the full
leap year rule concisely.

## Testing with `unittest`

Python's built-in `unittest` module lets you verify your code automatically:

```python
import unittest

class TestGreeting(unittest.TestCase):
    def test_returns_hello_world(self):
        self.assertEqual(greeting(), "Hello, World!")

if __name__ == "__main__":
    unittest.main()
```

- Test classes inherit from `unittest.TestCase`.
- Test methods start with `test_`.
- `assertEqual`, `assertTrue`, `assertFalse`, etc. check expected results.
- `if __name__ == "__main__"` ensures tests run only when the file is
  executed directly.

## Putting It Together

A well-structured Python file separates **logic** into functions and keeps
**tests** alongside the code:

```python
def greeting():
    return "Hello, World!"

def greet(name):
    return f"Hello, {name}!"

# --- Tests ---
import unittest

class TestGreeting(unittest.TestCase):
    def test_hello_world(self):
        self.assertEqual(greeting(), "Hello, World!")

    def test_greet_name(self):
        self.assertEqual(greet("Alice"), "Hello, Alice!")

if __name__ == "__main__":
    unittest.main()
```

This pattern — small focused functions verified by automated tests — is the
foundation of professional software development.