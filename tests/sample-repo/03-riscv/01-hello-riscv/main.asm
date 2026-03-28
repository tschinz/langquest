# Exercise 01: Hello, RISC-V!
#
# Welcome to your first RISC-V assembly exercise!
#
# In RISC-V, computation happens by loading values into registers and
# operating on them with instructions.  In this exercise you will use
# three fundamental instructions:
#
#   li   rd, imm        — load an immediate (constant) value into rd
#   add  rd, rs1, rs2   — rd = rs1 + rs2  (register + register)
#   addi rd, rs1, imm   — rd = rs1 + imm  (register + immediate constant)
#
# Register aliases used here:
#   t0  = x5    t1  = x6    t2  = x7    t3  = x28
#
# -----------------------------------------------------------------------
# Your tasks:
#   1. Load the value 10 into t0
#   2. Load the value 32 into t1
#   3. Compute t2 = t0 + t1          (should equal 42 — the answer!)
#   4. Compute t3 = t2 + 100         (should equal 142)
# -----------------------------------------------------------------------

# EXPECT_REG: x5  10    # t0 = 10
# EXPECT_REG: x6  32    # t1 = 32
# EXPECT_REG: x7  42    # t2 = t0 + t1
# EXPECT_REG: x28 142   # t3 = t2 + 100

.text
.globl _start

_start:
    # TODO: Load 10 into t0
    #   Hint: li t0, 10
    addi t0, zero, 10
    # TODO: Load 32 into t1
    #   Hint: li t1, 32
    addi t1, zero, 32
    # TODO: Compute t2 = t0 + t1
    #   Hint: add t2, t0, t1
    add t2, t0, t1
    # TODO: Compute t3 = t2 + 100
    #   Hint: addi t3, t2, 100
    addi t3, t2, 100
