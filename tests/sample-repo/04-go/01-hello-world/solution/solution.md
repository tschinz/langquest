---
title    = "Hello, Go!"
hints    = [
    "Just return the string literal directly.",
    "In Go, use return to send a value back.",
    "Make sure punctuation and capitalization match exactly.",
]
keywords = []
---

## Explanation

Go functions use explicit `return` statements. The return type comes after the parameter list.

```go
package main

import "fmt"

// Greeting returns the classic greeting string.
// In Go, exported functions start with an uppercase letter.
func Greeting() string {
    return "Hello, World!"  // Return the exact string
}

func main() {
    // fmt.Println prints to stdout with a newline
    fmt.Println(Greeting())
}
```

**Key concepts:**
- `func Name() Type` - function returning a value
- `return` - explicitly returns a value (required in Go)
- Uppercase function names are exported (public)
- `fmt.Println` - prints to stdout
