#!/bin/bash
cargo bootimage
qemu-system-x86_64 -machine pc -m 128M -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-blog_os.bin -device nvme,serial=deadbeef,drive=nvme0 -drive file=disk.img,if=none,id=nvme0,format=raw -device usb-ehci,id=ehci -device usb-tablet,bus=ehci.0 -no-reboot -d cpu_reset
