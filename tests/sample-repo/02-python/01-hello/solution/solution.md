---
title    = "Hello, Python!"
hints    = [
    "greeting() just needs to return a string literal — no formatting required.",
    "For greet(name), use an f-string: f\"Hello, {name}!\" lets you embed variables directly.",
    "my_abs(n) only needs an `if` check: if n is negative, return -n; otherwise return n as-is.",
    "For is_leap_year, combine the three divisibility rules with `and`/`or`: (divisible by 4 AND not by 100) OR (divisible by 400).",
]
keywords = [
    "def",
    "return",
    "f-string",
    "if",
    "elif",
    "else",
    "modulo",
    "conditional",
    "boolean",
    "str",
]
---

This exercise introduces four core Python building blocks: function definitions, string formatting, conditional branching, and boolean logic.

**greeting()** is the simplest possible function — it returns a string literal with no parameters:

```
def greeting():
    return "Hello, World!"
```

There is nothing to compute; just make sure the spelling, capitalisation, and punctuation match exactly.

**greet(name)** demonstrates f-strings, the modern Python way to embed expressions inside strings. Prefix the string with `f` and place the variable inside curly braces:

```
def greet(name):
    return f"Hello, {name}!"
```

f-strings evaluate the expression at runtime and insert the result. They are preferred over `"Hello, " + name + "!"` (concatenation) or `"Hello, {}!".format(name)` because they are more readable and more efficient.

**my_abs(n)** is an exercise in conditional logic. The absolute value of a number is its distance from zero — always non-negative. The only case where the input differs from the output is when n is negative:

```
def my_abs(n):
    if n < 0:
        return -n
    return n
```

Negating a negative number produces a positive one. Zero and positive numbers pass through unchanged. Note that this also works for floats because Python's comparison operators and negation work uniformly across numeric types.

**is_leap_year(year)** combines three divisibility rules into a single boolean expression. The Gregorian leap year rules are:

1. Divisible by 4 → leap year  2. Except divisible by 100 → not a leap year  3. Except divisible by 400 → leap year  The cleanest way to express this is:

```
def is_leap_year(year):
    return (year % 4 == 0 and year % 100 != 0) or (year % 400 == 0)
```

The `%` (modulo) operator returns the remainder of integer division. If `year % 4 == 0`, the year is evenly divisible by 4. The parentheses make the precedence explicit: first check the "divisible by 4 but not 100" case, then the "divisible by 400" override. Python's `and`/`or` operators short-circuit, so the expression is also efficient — if the first branch is True, the second is never evaluated.
