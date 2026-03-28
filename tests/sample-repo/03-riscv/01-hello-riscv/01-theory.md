# Introduction to RISC-V Assembly

RISC-V (pronounced "risk-five") is an open-source instruction set architecture
(ISA) that has taken the computing world by storm. Unlike x86, which evolved
over decades into a complex beast, RISC-V was designed from scratch to be
**clean**, **modular**, and **free for anyone to implement**.

## Why Learn RISC-V?

- **Simplicity** — RISC-V has far fewer instructions than x86, making it easier
  to learn and reason about.
- **Growing ecosystem** — RISC-V chips are shipping in everything from
  microcontrollers to server CPUs.
- **Open standard** — anyone can build a RISC-V processor without licensing
  fees, which is why it's the ISA of choice in academia and increasingly in
  industry.
- **Clean design** — no legacy baggage means consistent, orthogonal instruction
  encoding.

## RISC vs. CISC

RISC-V is a **RISC** (Reduced Instruction Set Computer) architecture, in
contrast to x86 which is **CISC** (Complex Instruction Set Computer):

| Feature           | RISC (RISC-V)                  | CISC (x86)                        |
|-------------------|--------------------------------|-----------------------------------|
| Instruction count | Small, ~50 base instructions   | Large, 1000+ instructions         |
| Instruction size  | Fixed (32-bit)                 | Variable (1–15 bytes)             |
| Memory access     | Only via load/store            | Arithmetic can access memory      |
| Registers         | 32 general-purpose             | 16 general-purpose (x86-64)       |
| Design philosophy | Simple hardware, smart compiler| Smart hardware, simple compiler   |

## RISC-V Registers

RISC-V RV64 (64-bit) has **32 general-purpose registers**, each 64 bits wide.
Every register also has a conventional name that indicates its typical usage:

| Register   | ABI Name | Purpose / Convention                       |
|------------|----------|--------------------------------------------|
| `x0`       | `zero`   | Hardwired to zero (writes are ignored)     |
| `x1`       | `ra`     | Return address                             |
| `x2`       | `sp`     | Stack pointer                              |
| `x3`       | `gp`     | Global pointer                             |
| `x4`       | `tp`     | Thread pointer                             |
| `x5`–`x7`  | `t0`–`t2`| Temporary registers (caller-saved)        |
| `x8`       | `s0`/`fp`| Saved register / Frame pointer             |
| `x9`       | `s1`     | Saved register (callee-saved)              |
| `x10`–`x11`| `a0`–`a1`| Function arguments / return values        |
| `x12`–`x17`| `a2`–`a7`| Function arguments                        |
| `x18`–`x27`| `s2`–`s11`| Saved registers (callee-saved)           |
| `x28`–`x31`| `t3`–`t6`| Temporary registers (caller-saved)        |

### Key Registers for Syscalls

When making Linux system calls on RISC-V:

- **`a7`** — holds the **syscall number**
- **`a0`** — first argument (and return value)
- **`a1`** — second argument
- **`a2`** — third argument
- **`a3`–`a5`** — fourth through sixth arguments

This is different from x86-64, where the syscall number goes in `rax` and
arguments use `rdi`, `rsi`, `rdx`, etc.

## Program Sections

Like x86 assembly, RISC-V assembly programs are organized into sections:

```asm
    .data                       # Initialized data
msg:
    .asciz "Hello!\n"           # Null-terminated string
    .set msg_len, . - msg       # Calculate string length

    .text                       # Executable code
    .globl _start               # Export entry point
_start:
    # instructions go here
```

### Key Directives

| Directive      | Purpose                                        |
|----------------|------------------------------------------------|
| `.data`        | Start of the initialized data section          |
| `.text`        | Start of the code (text) section               |
| `.globl`       | Make a symbol visible to the linker            |
| `.asciz`       | Define a null-terminated ASCII string          |
| `.ascii`       | Define an ASCII string (no null terminator)    |
| `.byte`        | Define raw byte values                         |
| `.set`         | Define a constant (like `equ` in NASM)         |
| `.` (dot)      | Current address (like `$` in NASM)             |

**Note:** RISC-V assemblers (GNU `as`) use the AT&T-style `.directive` syntax
with `#` for comments, unlike NASM which uses `;` for comments.

## Basic Instructions

### Loading Values

```asm
li   a0, 42          # Load Immediate: a0 = 42
la   a1, msg         # Load Address: a1 = address of msg
```

- **`li`** (Load Immediate) — loads a constant value into a register.
- **`la`** (Load Address) — loads the address of a label into a register.

### Moving Between Registers

```asm
mv   t0, a0          # Move: t0 = a0 (actually addi t0, a0, 0)
```

`mv` is a **pseudo-instruction** — the assembler translates it into
`addi t0, a0, 0`. RISC-V has many convenient pseudo-instructions that
expand into one or more real instructions.

## Linux System Calls with `ecall`

On RISC-V Linux, system calls are invoked using the **`ecall`** instruction
(environment call). This is analogous to `syscall` on x86-64.

### Syscall Convention

1. Place the **syscall number** in `a7`.
2. Place **arguments** in `a0`, `a1`, `a2`, etc.
3. Execute `ecall`.
4. The **return value** appears in `a0`.

### The `write` Syscall (number 64)

```asm
# ssize_t write(int fd, const void *buf, size_t count)
li   a7, 64          # Syscall number for write
li   a0, 1           # File descriptor 1 = stdout
la   a1, msg         # Pointer to the message buffer
li   a2, 14          # Number of bytes to write
ecall                # Invoke the kernel
```

**Note:** The write syscall number on RISC-V Linux is **64**, not 1 like on
x86-64! RISC-V uses a different syscall numbering scheme.

### The `exit` Syscall (number 93)

```asm
# void exit(int status)
li   a7, 93          # Syscall number for exit
li   a0, 0           # Exit code 0 = success
ecall                # Program terminates here
```

The exit syscall number is **93** on RISC-V (versus 60 on x86-64).

### Syscall Number Quick Reference

| Syscall | Number (RISC-V) | Number (x86-64) | Arguments                  |
|---------|-----------------|-----------------|----------------------------|
| write   | 64              | 1               | fd, buf, count             |
| exit    | 93              | 60              | status                     |
| read    | 63              | 0               | fd, buf, count             |
| openat  | 56              | 257             | dirfd, path, flags, mode   |

## A Complete "Hello" Program

```asm
    .data
msg:
    .ascii "Hello!\n"
    .set msg_len, . - msg

    .text
    .globl _start
_start:
    # write(1, msg, msg_len)
    li   a7, 64              # syscall: write
    li   a0, 1               # fd: stdout
    la   a1, msg             # buf: address of msg
    li   a2, msg_len         # count: length of msg
    ecall

    # exit(0)
    li   a7, 93              # syscall: exit
    li   a0, 0               # status: success
    ecall
```

The structure is clean and readable — load the syscall number, set up
arguments in order, then `ecall`. This regularity is one of the great
strengths of RISC-V.

## Comparing x86-64 and RISC-V

If you've already done the x86-64 exercises, here's a handy comparison:

| Concept         | x86-64                        | RISC-V                     |
|-----------------|-------------------------------|----------------------------|
| Syscall trigger | `syscall`                     | `ecall`                    |
| Syscall number  | `rax`                         | `a7`                       |
| 1st argument    | `rdi`                         | `a0`                       |
| 2nd argument    | `rsi`                         | `a1`                       |
| 3rd argument    | `rdx`                         | `a2`                       |
| Load immediate  | `mov rax, 42`                 | `li a0, 42`                |
| Load address    | `mov rsi, msg` / `lea rsi, [msg]` | `la a1, msg`          |
| Entry point     | `global _start` / `_start:`   | `.globl _start` / `_start:`|
| Comment char    | `;`                           | `#`                        |

## Building and Running

RISC-V assembly programs can be assembled and linked using the GNU toolchain:

```sh
riscv64-linux-gnu-as main.asm -o .lq_main.o
riscv64-linux-gnu-ld .lq_main.o -o .lq_main
```

If running on a non-RISC-V host, you can use **QEMU** user-mode emulation:

```sh
qemu-riscv64 ./.lq_main
```

This makes it possible to develop and test RISC-V programs on any Linux machine.