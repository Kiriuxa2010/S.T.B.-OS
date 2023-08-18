// same thing as in /src/

#![no_std]
#![feature(asm)]

use crate::println;
use core::arch::asm;

pub fn get_cpu_name() -> Option<[u8; 48]> {
    let mut cpu_name: [u8; 48] = [0; 48];
    unsafe {
        asm!(
            "mov eax, 0x80000002",
            "cpuid",
            "mov [rdi], eax",
            "mov [rdi + 4], ebx",
            "mov [rdi + 8], ecx",
            "mov [rdi + 12], edx",
            "mov eax, 0x80000003",
            "cpuid",
            "mov [rdi + 16], eax",
            "mov [rdi + 20], ebx",
            "mov [rdi + 24], ecx",
            "mov [rdi + 28], edx",
            "mov eax, 0x80000004",
            "cpuid",
            "mov [rdi + 32], eax",
            "mov [rdi + 36], ebx",
            "mov [rdi + 40], ecx",
            "mov [rdi + 44], edx",
            in("rdi") &mut cpu_name as *mut _,
        );
    }

    Some(cpu_name)
}

pub fn print_cpu_name(cpu_name: &[u8; 48]) {
    let name_str = core::str::from_utf8(cpu_name).unwrap_or("<Invalid UTF-8>");
    println!(" CPU Name: {}", name_str.trim());
}
