# Functions in Go

Go functions are declared with `func` and can return values.

## Basic Function

```go
func GetNumber() int {
    return 42
}
```

## Return Types

The return type comes **after** the parameter list:

```go
func Add(a, b int) int {
    return a + b
}
```

## Exported vs Unexported

In Go, **capitalization matters**:

- `Greeting` — uppercase first letter = **exported** (public)
- `greeting` — lowercase first letter = unexported (private)

## String Type

The `string` type holds text:

```go
func GetText() string {
    return "some text"
}
```
