target extended-remote :3333
# target remote localhost:3333

set confirm off

set mem inaccessible-by-default off
set architecture riscv:rv32
set remotetimeout unlimited

# print demangled symbols
set print asm-demangle on

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
# break HardFault
break rust_begin_unwind

# *try* to stop at the user entry point (it might be gone due to inlining)
break main

# monitor reset halt

monitor halt
monitor poll

# load
# stepi
