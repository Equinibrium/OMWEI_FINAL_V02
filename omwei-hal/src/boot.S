.section .text.boot
.global _start
_start:
    # Setup stack for Hart 0 - USE CORRECT SYMBOL
    la sp, __hart0_stack_end
    # Jump to Rust kmain
    call kmain
park:
    wfi
    j park
