#define EL0 0b00
#define EL1 0b01
#define EL2 0b10
#define EL3 0b11

.section .text.init

.global _start
_start:
    // read cpu affinity, start core 0, halt rest
    mrs     x1, MPIDR_EL1
    and     x1, x1, #3
    cbz     x1, setup

halt:
    // core affinity != 0, halt it
    wfe
    b       halt

setup:
    // store the desired EL1 stack pointer in x1
    adr     x1, _start

    // read the current exception level into x0 (ref: C5.2.1)
    mrs     x0, CurrentEL
    and     x0, x0, #0b1100
    lsr     x0, x0, #2

switch_to_el2:
    // switch to EL2 if we're in EL3. otherwise switch to EL1
    cmp     x0, EL3
    bne     switch_to_el1

    // set-up SCR_EL3 (bits 0, 4, 5, 7, 8, 10) (A53: 4.3.42)
    mov     x2, #0x5b1
    msr     SCR_EL3, x2

    // set-up SPSR and PL switch! (bits 0, 3, 6, 7, 8, 9) (ref: C5.2.20)
    mov     x2, #0x3c9
    msr     SPSR_EL3, x2
    adr     x2, switch_to_el1
    msr     ELR_EL3, x2
    eret

switch_to_el1:
    // switch to EL1 if we're not already in EL1. otherwise continue with start
    cmp     x0, EL1
    beq     set_stack

    // set the stack-pointer for EL1
    msr     SP_EL1, x1

    // enable CNTP for EL1/EL0 (ref: D7.5.2, D7.5.13)
    // NOTE: This doesn't actually enable the counter stream.
    mrs     x0, CNTHCTL_EL2
    orr     x0, x0, #0b11
    msr     CNTHCTL_EL2, x0
    msr     CNTVOFF_EL2, xzr

    // enable AArch64 in EL1 (A53: 4.3.36)
    mov     x0, #(1 << 31)      // Enable AArch64 for EL1
    orr     x0, x0, #(1 << 1)   // RES1 on A-53
    msr     HCR_EL2, x0
    mrs     x0, HCR_EL2

    // enable floating point and SVE (SIMD) (A53: 4.3.38, 4.3.34)
    msr     CPTR_EL2, xzr     // don't trap accessing SVE registers
    mrs     x0, CPACR_EL1
    orr     x0, x0, #(0b11 << 20)
    msr     CPACR_EL1, x0

    // Set SCTLR to known state (RES1: 11, 20, 22, 23, 28, 29) (A53: 4.3.30)
    mov     x2, #0x0800
    movk    x2, #0x30d0, lsl #16
    msr     SCTLR_EL1, x2

    // set up exception handlers
    adr     x0, _vectors
    msr     VBAR_EL1, x0

    // change execution level to EL1 (ref: C5.2.19)
    mov     x2, #0x3c5
    msr     SPSR_EL2, x2
    // Return to EL1 at `set_stack`
    // Set return address at `set_stack`
    adr     x0, set_stack
    msr     ELR_EL2, x0
    // Return to EL1
    eret

set_stack:
    // set the current stack pointer
    mov     sp, x1

zero_bss:
    // load the start address and number of bytes in BSS section
    ldr     x1, =__bss_start
    ldr     x2, =__bss_length

zero_bss_loop:
    // zero out the BSS section, 64-bits at a time
    cbz     x2, go_kmain
    str     xzr, [x1], #8
    sub     x2, x2, #8
    cbnz    x2, zero_bss_loop

go_kmain:
    // jump to kmain, which shouldn't return. halt if it does
    bl      kmain
    b       halt

context_save:
    // Set up the trap frame on the stack
    // The trap frame contains all the information
    // needed to restore the execution state
    // The total size is 800 bytes.
    // 64-bit registers
    // It is our responsibility to save `x0` and `lr` __again__
    // the ones saved in `HANDLER` has no relationship with ours
    // `context_save` is also a routine, it MUST save `lr`
    // if it wants to call some other routines.
    stp     x0, x30, [SP, #-16]!
    stp     x1, x2, [SP, #-16]!
    stp     x3, x4, [SP, #-16]!
    stp     x5, x6, [SP, #-16]!
    stp     x7, x8, [SP, #-16]!
    stp     x9, x10, [SP, #-16]!
    stp     x11, x12, [SP, #-16]!
    stp     x13, x14, [SP, #-16]!
    stp     x15, x16, [SP, #-16]!
    stp     x17, x18, [SP, #-16]!
    stp     x19, x20, [SP, #-16]!
    stp     x21, x22, [SP, #-16]!
    stp     x23, x24, [SP, #-16]!
    stp     x25, x26, [SP, #-16]!
    stp     x27, x28, [SP, #-16]!
    str     x29, [SP, #-16]! // contains a reserved field of 8 bytes

    // 128-bit registers
    stp     q0, q31, [SP, #-32]!
    stp     q1, q2, [SP, #-32]!
    stp     q3, q4, [SP, #-32]!
    stp     q5, q6, [SP, #-32]!
    stp     q7, q8, [SP, #-32]!
    stp     q9, q10, [SP, #-32]!
    stp     q11, q12, [SP, #-32]!
    stp     q13, q14, [SP, #-32]!
    stp     q15, q16, [SP, #-32]!
    stp     q17, q18, [SP, #-32]!
    stp     q19, q20, [SP, #-32]!
    stp     q21, q22, [SP, #-32]!
    stp     q23, q24, [SP, #-32]!
    stp     q25, q26, [SP, #-32]!
    stp     q27, q28, [SP, #-32]!
    stp     q29, q30, [SP, #-32]!

    // Special registers
    mrs     x19, SPSR_EL1          // PSTATE
    mrs     x20, ELR_EL1           // Program Counter
    stp     x19, x20, [SP, #-16]!
    mrs     x19, SP_EL0            // Stack pointer (of EL0)
    mrs     x20, TPIDR_EL0         // Thread ID (of EL0)
    stp     x19, x20, [SP, #-16]!

    mrs     x1, ESR_EL1           // Exception Syndrome
    mov     x2, SP
    bl      handle_exception
    mov     x0, SP                // Argument for context_restore
    mov     x1, xzr
    add     SP, SP, #800          // Restore the SP before falling into context_restore

.global context_restore
context_restore:
    // Restore the trap frame to the registers
    // The first argument (x0) should be pointer to the trap frame
    // Special registers
    ldp     x19, x20, [x0], #16
    msr     SP_EL0, x19
    msr     TPIDR_EL0, x20
    ldp     x19, x20, [x0], #16
    msr     SPSR_EL1, x19
    msr     ELR_EL1, x20

    // 128-bit registers
    ldp     q29, q30, [x0], #32
    ldp     q27, q28, [x0], #32
    ldp     q25, q26, [x0], #32
    ldp     q23, q24, [x0], #32
    ldp     q21, q22, [x0], #32
    ldp     q19, q20, [x0], #32
    ldp     q17, q18, [x0], #32
    ldp     q15, q16, [x0], #32
    ldp     q13, q14, [x0], #32
    ldp     q11, q12, [x0], #32
    ldp     q9, q10, [x0], #32
    ldp     q7, q8, [x0], #32
    ldp     q6, q5, [x0], #32
    ldp     q4, q3, [x0], #32
    ldp     q1, q2, [x0], #32
    ldp     q0, q31, [x0], #32

    // 64-bit registers
    ldr     x29, [x0], #16
    ldp     x27, x28, [x0], #16
    ldp     x25, x26, [x0], #16
    ldp     x23, x24, [x0], #16
    ldp     x21, x22, [x0], #16
    ldp     x19, x20, [x0], #16
    ldp     x17, x18, [x0], #16
    ldp     x15, x16, [x0], #16
    ldp     x13, x14, [x0], #16
    ldp     x11, x12, [x0], #16
    ldp     x9, x10, [x0], #16
    ldp     x7, x8, [x0], #16
    ldp     x6, x5, [x0], #16
    ldp     x4, x3, [x0], #16

    // If `x1` is 0, we do a normal return
    // Otherwise we directly call `eret`
    // we do this comparison before overwriting `x1`
    cmp     x1, xzr
    beq     context_restore_return

    // Restore the remaining registers
    ldp     x1, x2, [x0], #16
    ldp     x0, x30, [x0] // Finally overwrite x0 with restored context

    // Reset SP to _start
    // x30 is not used anyway if we `eret`, just use it for intermediate values
    adr     x30, _start
    mov     SP, x30

    // Switch level!
    eret

context_restore_return:
    // Restore the remaining registers
    ldp     x1, x2, [x0], #16
    ldp     x0, x30, [x0] // Finally overwrite x0 with restored context
    ret

#define HANDLER(source, kind) \
    .align 7; \
    stp     x30, x0, [SP, #-16]!; \
    mov     x0, ##source; \
    movk    x0, ##kind, LSL #16; \
    bl      context_save; \
    ldp     x30, x0, [SP], #16; \
    eret

.align 11
_vectors:
    HANDLER(0, 0)
    HANDLER(0, 1)
    HANDLER(0, 2)
    HANDLER(0, 3)
    HANDLER(1, 0)
    HANDLER(1, 1)
    HANDLER(1, 2)
    HANDLER(1, 3)
    HANDLER(2, 0)
    HANDLER(2, 1)
    HANDLER(2, 2)
    HANDLER(2, 3)
    HANDLER(3, 0)
    HANDLER(3, 1)
    HANDLER(3, 2)
    HANDLER(3, 3)