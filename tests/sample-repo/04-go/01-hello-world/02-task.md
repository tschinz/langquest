---
id          = "hello_go"
name        = "Hello, Go!"
language    = "go"
difficulty  = 1
description = "Write your first Go function that returns a greeting string."
topics      = ["functions", "strings", "fmt", "basic_syntax"]
---

# Hello, Go!

## Objective

Make the program print **exactly** `Hello, World!` to standard output by
implementing the `Greeting` function.

## Instructions

1. Open `main.go` and find the `Greeting` function.
2. Replace the placeholder `return ""` with an expression that returns the
   string `"Hello, World!"`.
3. The `main` function already calls `Greeting()` and prints the result — you
   only need to fix `Greeting`.

## Requirements

- `Greeting` must return a `string` with the value `"Hello, World!"`.
- Do **not** modify `main` or the test file `main_test.go`.

## Running

```sh
# Run the program
go run main.go

# Run the tests to verify your solution
go test -v .
```

## Expected Output

```
Hello, World!
```
