# EXPECT_REG: x5  0xA
# EXPECT_REG: x6  0x20
# EXPECT_REG: x7  42
# EXPECT_REG: x28 142

_start:
    # TODO: Load 10 into t0 (x5)
    addi t0, zero, 10
    # TODO: Load 32 into t1 (x6)
    addi t1, zero, 32
    # TODO: Compute t2 = t0 + t1
    add t2, t0, t1
    # TODO: Compute t3 = t2 + 100
    addi t3, t2, 100
