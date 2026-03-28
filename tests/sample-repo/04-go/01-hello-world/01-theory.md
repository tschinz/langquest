# Hello, World!

Every programmer's journey begins with a simple greeting. In Go, this tradition
is no different — and even this tiny program introduces several important
concepts of the language.

## Packages and the `main` Package

Every Go source file starts with a **package declaration**:

```go
package main
```

- `package main` is special — it marks the file as an **executable program**
  rather than a library.
- The `main` package must contain exactly one `main()` function, which is the
  **entry point** of the program.

## Imports

Go uses explicit imports to bring in external packages:

```go
import "fmt"
```

- `fmt` is the standard library package for **formatted I/O** (short for
  *format*).
- Unused imports are a **compile error** in Go — the compiler enforces clean
  code.

## The `fmt.Println` Function

To print a line to the console:

```go
fmt.Println("Hello, World!")
```

- `fmt.Println` writes the arguments to stdout followed by a newline.
- The package name (`fmt`) is always part of the call — there is no implicit
  namespace in Go.

## Functions

Functions are declared with the `func` keyword:

```go
func greet() string {
    return "Hello, World!"
}
```

- `func` introduces a function definition.
- The return type comes **after** the parameter list, before the opening brace.
- Go uses explicit `return` statements (unlike Rust's expression-based returns).

## Putting It Together

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

Splitting logic into small, named functions makes code easier to **test** and
**reuse** — which is exactly why this exercise asks you to implement
`Greeting()` separately from `main()`.

## Testing in Go

Go has a built-in test framework in the `testing` package. Test files end in
`_test.go` and test functions follow the naming convention `TestXxx`:

```go
func TestGreeting(t *testing.T) {
    got := Greeting()
    want := "Hello, World!"
    if got != want {
        t.Errorf("Greeting() = %q, want %q", got, want)
    }
}
```

Run tests with:

```sh
go test -v .
```

The `-v` flag shows each test's pass/fail status individually.