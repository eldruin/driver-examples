target remote :3333

# print demangled symbols by default
set print asm-demangle on

monitor arm semihosting enable

load
continue
