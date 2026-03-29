# Functions in Python

Functions are defined using the `def` keyword and can return values.

## Basic Function

```python
def get_number():
    return 42
```

## Parameters

Functions can accept parameters:

```python
def double(x):
    return x * 2
```

## Return Values

Use `return` to send a value back to the caller:

```python
def add(a, b):
    return a + b

result = add(2, 3)  # result = 5
```

If no `return` is specified, the function returns `None`.