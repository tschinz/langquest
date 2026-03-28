# Exercise 01: Hello, RISC-V! — Solution
#
# EXPECT_REG: x5  10    # t0 = 10
# EXPECT_REG: x6  32    # t1 = 32
# EXPECT_REG: x7  42    # t2 = t0 + t1
# EXPECT_REG: x28 142   # t3 = t2 + 100

.text
.globl _start

_start:
    li   t0, 10           # t0 = 10
    li   t1, 32           # t1 = 32
    add  t2, t0, t1       # t2 = 10 + 32 = 42
    addi t3, t2, 100      # t3 = 42 + 100 = 142