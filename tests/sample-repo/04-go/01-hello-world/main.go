// Exercise 01: Hello, Go!
//
// Your first Go program! In this exercise you will implement a simple
// function that returns a greeting string.
//
// TODO: Implement the Greeting function so that it returns the string
//       "Hello, World!" exactly (including punctuation and capitalisation).

package main

import "fmt"

// Greeting returns the classic "Hello, World!" greeting.
// TODO: implement this function
func Greeting() string {
  return "Hello, World!"
}

func main() {
  fmt.Println(Greeting())
}
