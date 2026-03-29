# Solution: Register Arithmetic
#
# This program demonstrates basic RISC-V instructions:
# - li (load immediate): loads a constant into a register
# - add: adds two registers
# - addi: adds a register and an immediate value

# EXPECT_REG: x5  10
# EXPECT_REG: x6  32
# EXPECT_REG: x7  42
# EXPECT_REG: x28 142

_start:
    # Load immediate values into registers
    li   t0, 10         # t0 (x5) = 10
    li   t1, 32         # t1 (x6) = 32

    # Add two registers: t2 = t0 + t1
    add  t2, t0, t1     # t2 (x7) = 10 + 32 = 42

    # Add register and immediate: t3 = t2 + 100
    addi t3, t2, 100    # t3 (x28) = 42 + 100 = 142
