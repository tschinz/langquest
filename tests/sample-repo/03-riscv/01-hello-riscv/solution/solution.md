---
title    = "Register Arithmetic"
hints    = [
    "Use li (load immediate) to put a constant into a register.",
    "Use add to sum two registers: add rd, rs1, rs2",
    "Use addi to add an immediate value: addi rd, rs1, imm",
]
keywords = []
---

## Explanation

RISC-V uses simple load and arithmetic instructions to manipulate register values.

```asm

_start:
    # Load immediate values into registers
    li   t0, 10           # t0 = 10 (load immediate)
    li   t1, 32           # t1 = 32

    # Add two registers: rd = rs1 + rs2
    add  t2, t0, t1       # t2 = 10 + 32 = 42

    # Add register + immediate: rd = rs1 + imm
    addi t3, t2, 100      # t3 = 42 + 100 = 142
```

**Key concepts:**
- `li rd, imm` - load an immediate (constant) value into register rd
- `add rd, rs1, rs2` - add two registers, store result in rd
- `addi rd, rs1, imm` - add a register and immediate value
- `t0`–`t6` - temporary registers (caller-saved)
