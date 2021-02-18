#! /bin/bash

if [[ $1 == "release" ]]; then
    cargo objcopy --release --bin $2 -- -O binary target/riscv64gc-unknown-none-elf/release/$2.bin
    kflash -S -t -p /dev/tty.usbserial-14410 -B dan -b 150000 target/riscv64gc-unknown-none-elf/release/$2.bin
else 
    cargo objcopy --bin $2 -- -O binary target/riscv64gc-unknown-none-elf/debug/$2.bin
    kflash -S -t -p /dev/tty.usbserial-14410 -B dan -b 150000 target/riscv64gc-unknown-none-elf/debug/$2.bin
fi
