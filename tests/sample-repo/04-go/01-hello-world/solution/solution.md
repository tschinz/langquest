---
title    = "Hello, Go!"
hints    = [
    "The function signature already tells you the return type: string",
    "A string literal like \"Hello, World!\" is all you need — no imports required inside the function",
    "In Go, return statements are explicit: write `return \"Hello, World!\"`",
    "Make sure the capitalisation and punctuation match exactly: capital H, capital W, comma, space, exclamation mark",
]
keywords = [
    "func",
    "string",
    "return",
    "package main",
    "import",
    "fmt.Println",
    "string literal",
]
---

Every Go executable lives in `package main` and its entry point is the `main()` function. In this exercise the greeting logic is extracted into its own named function so it can be tested independently — a pattern you will use throughout the course.

## The `Greeting` Function

The solution is simply:

```go
func Greeting() string {
    return "Hello, World!"
}
```

A few things to note:

- The return type `string` appears **after** the parameter list (which is empty here). This is the opposite of C-family languages where the type comes first.
- Go uses an explicit `return` statement. Unlike Rust, the last expression in a block is **not** implicitly returned — you must write `return`.
- The function name starts with a capital letter (`Greeting`, not `greeting`). In Go, an **uppercase first letter makes an identifier exported** (visible outside the package). This matters when your code is used as a library; for `package main` it is a convention rather than a strict requirement, but it is good practice.

## The Full Program

```go
package main

import "fmt"

func Greeting() string {
    return "Hello, World!"
}

func main() {
    fmt.Println(Greeting())
}
```

`fmt.Println` writes its arguments to stdout followed by a newline. The package qualifier (`fmt.`) is always required — Go has no implicit namespace imports.

## How `go test` Works

The test file `main_test.go` is in the same package (`package main`), which means it has direct access to all identifiers in `main.go` without any imports. Running:

```sh
go test -v .
```

compiles both files together, executes every function whose name starts with `Test`, and reports a `PASS` or `FAIL` line for each one. The `-v` flag enables verbose output so you can see every individual test result.