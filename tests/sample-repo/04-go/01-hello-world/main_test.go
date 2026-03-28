// Tests for Exercise 01: Hello, Go!
//
// Do NOT modify this file — it is used to verify your solution.

package main

import (
  "strings"
  "testing"
)

func TestGreeting(t *testing.T) {
  got := Greeting()
  want := "Hello, World!"
  if got != want {
  t.Errorf("Greeting() = %q, want %q", got, want)
  }
}

func TestGreetingNotEmpty(t *testing.T) {
  if Greeting() == "" {
  t.Error("Greeting() should not return an empty string")
  }
}

func TestGreetingStartsWithHello(t *testing.T) {
  got := Greeting()
  if !strings.HasPrefix(got, "Hello") {
  t.Errorf("Greeting() = %q, should start with \"Hello\"", got)
  }
}

func TestGreetingEndsWithExclamation(t *testing.T) {
  got := Greeting()
  if !strings.HasSuffix(got, "!") {
  t.Errorf("Greeting() = %q, should end with '!'", got)
  }
}

func TestGreetingContainsWorld(t *testing.T) {
  got := Greeting()
  if !strings.Contains(got, "World") {
  t.Errorf("Greeting() = %q, should contain \"World\"", got)
  }
}
