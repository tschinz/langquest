---
title    = "Hello, RISC-V!"
hints    = [
    "The .data section already has msg and msg_len defined for you — focus on the .text section where you need to set up registers for the ecall instruction.",
    "For the write ecall: set a7=64 (syscall number), a0=1 (stdout file descriptor), a1=address of msg (use `la a1, msg`), a2=msg_len (use `li a2, 14`), then execute `ecall`.",
    "For the exit ecall: set a7=93 (syscall number), a0=0 (exit code), then execute `ecall`.",
    "Remember to perform the write ecall BEFORE the exit ecall — once the program exits, no more code executes! The order of syscalls matters.",
]
keywords = [
    "ecall",
    ".data",
    ".text",
    "_start",
    ".globl",
    "msg",
    "a0",
    "a1",
    "a2",
    "a7",
    "li",
    "la",
    "write",
    "exit",
]
---

This exercise introduces RISC-V assembly programming on Linux. The program is structured into two sections, similar to x86-64 but with different syntax and conventions.

**`.data` section** — This is where we store initialized data. The message string is defined using the `.asciz` directive, which automatically appends a null terminator (though we rely on an explicit length rather than null termination for the write syscall). The newline is included via the `\n` escape sequence.

**`.text` section** — This contains the executable code. `.globl _start` exports the `_start` symbol so the linker can find the entry point.

**RISC-V Linux syscalls** are invoked using the `ecall` instruction. Before executing `ecall`, you place the syscall number in register `a7` and the arguments in registers `a0` through `a5`:

1. **write(fd, buf, count)** — syscall 64: `a7=64`, `a0=1` (stdout), `a1=address of msg` (loaded with `la`), `a2=14` (byte count including the newline).
2. **exit(code)** — syscall 93: `a7=93`, `a0=0` (success exit code).

Key RISC-V instructions used:
- `li reg, imm` — Load Immediate: loads a constant value into a register.
- `la reg, label` — Load Address: loads the memory address of a label into a register. This is a pseudo-instruction that the assembler expands into one or two real instructions depending on the address.
- `ecall` — Environment Call: triggers a trap to the operating system, which reads a7 to determine which syscall to execute.

Unlike x86-64 where `mov` is the universal data-movement instruction, RISC-V separates loading immediates (`li`) from loading addresses (`la`) and from register-to-register moves (`mv`). This is characteristic of RISC architectures — each instruction does one simple thing.

The program is assembled and linked with a RISC-V toolchain:
```
riscv64-linux-gnu-as main.asm -o .lq_main.o
riscv64-linux-gnu-ld .lq_main.o -o .lq_main
```

On a RISC-V system (or under emulation with QEMU), the resulting binary will print "Hello, RISC-V!" followed by a newline and exit with code 0.
