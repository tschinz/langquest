// Solution: Hello, Go!
//
// This program demonstrates the basic structure of a Go executable:
// - package main: marks this as an executable (not a library)
// - import: brings in external packages
// - func main(): the entry point of the program

package main

import "fmt"

// Greeting returns the classic "Hello, World!" greeting.
// In Go, functions that start with an uppercase letter are exported
// (visible outside the package).
func Greeting() string {
  return "Hello, World!"
}

func main() {
  // Print the greeting to stdout with a newline
  fmt.Println(Greeting())
}
