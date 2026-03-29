package main

import "testing"

func TestGreeting(t *testing.T) {
  got := Greeting()
  want := "Hello, World!"
  if got != want {
  t.Errorf("Greeting() = %q, want %q", got, want)
  }
}
