# Test-UEFI-Bootloader
This Is The Start Of A UEFI Bootloader 

Currently This Is A UEFI Bootable App And Just Needs To Load The Kernel And Set A Few Things Up To Make This A Bootloader

## This Is The Command For Running The .gdt File With QUEMU You Need The .fd File Also:

qemu-system-x86_64 -drive format=raw,file=uefi_app.gdt -bios OVMF-pure-efi.fd


## These Are The Build Commands:

cargo build --target x86_64-unknown-uefi -Z build-std=core,alloc -Z build-std-features=compiler-builtins-mem

and

cargo run --package disk_image -- target/x86_64-unknown-uefi/debug/uefi_app.efi


# Credit
All Credit Goes To Philipp Oppermann 
[https://github.com/phil-opp/blog_os/blob/edition-3/blog/content/edition-3/posts/02-booting/uefi/index.md](https://github.com/phil-opp/blog_os/blob/edition-3/blog/content/edition-3/posts/02-booting/uefi/index.md)
