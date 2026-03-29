# RISC-V Basics

RISC-V is an open instruction set architecture with 32 general-purpose registers.

## Registers

Registers are named `x0` through `x31`, with conventional aliases:

| Register | Alias | Purpose |
|----------|-------|---------|
| `x0` | `zero` | Always zero (writes ignored) |
| `x5`–`x7` | `t0`–`t2` | Temporary values |
| `x28`–`x31` | `t3`–`t6` | More temporaries |

## Loading Values

Use `li` (load immediate) to put a constant into a register:

```asm
li t0, 42       # t0 = 42
```

## Arithmetic

Basic integer operations:

```asm
add  t2, t0, t1    # t2 = t0 + t1 (register + register)
addi t3, t2, 10    # t3 = t2 + 10 (register + immediate)
```

The `add` instruction adds two registers. The `addi` instruction adds a register and a constant.