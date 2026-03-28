---
id          = "hello_riscv"
name        = "Hello, RISC-V!"
language    = "riscv"
difficulty  = 3
description = "Write a RISC-V assembly program that prints 'Hello, RISC-V!' using Linux ecall syscalls."
topics      = ["riscv", "syscalls", "registers", "ecall"]
---

# Hello, RISC-V!

## Objective

Write a RISC-V assembly program that prints **exactly** `Hello, RISC-V!`
followed by a newline to standard output, then exits cleanly with exit code **0**.

## Instructions

1. Open `main.asm` and find the `_start` label in the `.text` section.
2. The message string `msg` and its length `msg_len` are already defined for you
   in the `.data` section — you do **not** need to modify the data section.
3. Implement the **write** ecall to print the message to stdout:
   - Load the syscall number for `write` (**64**) into register `a7`.
   - Load the file descriptor for stdout (**1**) into register `a0`.
   - Load the **address** of `msg` into register `a1`.
   - Load `msg_len` (the length of the string) into register `a2`.
   - Execute the `ecall` instruction.
4. Implement the **exit** ecall to terminate the program:
   - Load the syscall number for `exit` (**93**) into register `a7`.
   - Load exit code **0** into register `a0`.
   - Execute the `ecall` instruction.

## RISC-V Linux Syscall Convention

On RISC-V Linux, system calls are invoked with the `ecall` instruction.
Arguments are passed in registers:

| Purpose       | Register |
|---------------|----------|
| Syscall number| `a7`     |
| 1st argument  | `a0`     |
| 2nd argument  | `a1`     |
| 3rd argument  | `a2`     |
| 4th argument  | `a3`     |
| Return value  | `a0`     |

### Relevant Syscalls

| Syscall | Number | a0             | a1             | a2           |
|---------|--------|----------------|----------------|--------------|
| write   | 64     | fd (1=stdout)  | buffer address | buffer length|
| exit    | 93     | exit code      | —              | —            |

## Requirements

- The program must output `Hello, RISC-V!` followed by a newline to **stdout** (file descriptor 1).
- The program must exit with code **0**.
- You must use the `write` ecall (number **64**) and the `exit` ecall (number **93**).
- Do **not** modify the `.data` section — `msg` and `msg_len` are already defined.
- Do **not** remove or modify the `; EXPECT_*` directives at the top of the file.

## Loading Addresses

To load the address of a label in RISC-V, use the `la` (load address)
pseudo-instruction:

```asm
la a1, msg        # Load the address of msg into a1
```

To load immediate values, use `li` (load immediate):

```asm
li a7, 64         # Load the value 64 into a7
```

## Expected Output

```
Hello, RISC-V!
```
