---
id          = "hello_riscv"
name        = "Register Arithmetic"
language    = "riscv"
difficulty  = 1
description = "Load values into registers and perform basic arithmetic."
topics      = ["registers", "li", "add", "addi"]
---

# Register Arithmetic

Use RISC-V instructions to load values and perform arithmetic operations.

## Tasks

1. Load `10` into register `t0`
2. Load `32` into register `t1`
3. Compute `t2 = t0 + t1` (should be 42)
4. Compute `t3 = t2 + 100` (should be 142)

## Expected Result

After execution, the registers should contain:

- `t0` = 10
- `t1` = 32
- `t2` = 42
- `t3` = 142