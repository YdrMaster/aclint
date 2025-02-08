#![no_std]
#![feature(naked_functions)]
#![deny(warnings)]

use core::{arch::naked_asm, cell::UnsafeCell, mem::size_of};

#[repr(transparent)]
pub struct MTIME(UnsafeCell<u64>);

#[repr(transparent)]
pub struct MTIMECMP(UnsafeCell<u64>);

#[repr(transparent)]
pub struct MSIP(UnsafeCell<u32>);

#[repr(transparent)]
pub struct SETSSIP(UnsafeCell<u32>);

#[repr(transparent)]
pub struct MTIMER([MTIMECMP; 4095]);

#[repr(transparent)]
pub struct MSWI([MSIP; 4095]);

#[repr(transparent)]
pub struct SSWI([SETSSIP; 4095]);

#[repr(C)]
pub struct SifiveClint {
    mswi: MSWI,
    reserve: u32,
    mtimer: MTIMER,
    mtime: MTIME,
}

impl SifiveClint {
    const MTIMER_OFFSET: usize = size_of::<MSWI>() + size_of::<u32>();
    const MTIME_OFFSET: usize = Self::MTIMER_OFFSET + size_of::<MTIMER>();

    #[inline]
    pub fn read_mtime(&self) -> u64 {
        unsafe { self.mtime.0.get().read_volatile() }
    }

    #[inline]
    pub fn write_mtime(&self, val: u64) {
        unsafe { self.mtime.0.get().write_volatile(val) }
    }

    #[inline]
    pub fn read_mtimecmp(&self, hart_idx: usize) -> u64 {
        unsafe { self.mtimer.0[hart_idx].0.get().read_volatile() }
    }

    #[inline]
    pub fn write_mtimecmp(&self, hart_idx: usize, val: u64) {
        unsafe { self.mtimer.0[hart_idx].0.get().write_volatile(val) }
    }

    #[inline]
    pub fn read_msip(&self, hart_idx: usize) -> bool {
        unsafe { self.mswi.0[hart_idx].0.get().read_volatile() != 0 }
    }

    #[inline]
    pub fn set_msip(&self, hart_idx: usize) {
        unsafe { self.mswi.0[hart_idx].0.get().write_volatile(1) }
    }

    #[inline]
    pub fn clear_msip(&self, hart_idx: usize) {
        unsafe { self.mswi.0[hart_idx].0.get().write_volatile(0) }
    }
}

impl SifiveClint {
    #[naked]
    pub extern "C" fn read_mtime_naked(&self) -> u64 {
        unsafe {
            naked_asm!(
                "   addi sp, sp, -8
                    sd   a1, (sp)

                    li   a1, {offset}
                    add  a0, a0, a1

                    ld   a1, (sp)
                    addi sp, sp,  8

                    ld   a0, (a0)
                    ret
                ",
                offset = const Self::MTIME_OFFSET,
            )
        }
    }

    #[naked]
    pub extern "C" fn write_mtime_naked(&self, val: u64) -> u64 {
        unsafe {
            naked_asm!(
                "   addi sp, sp, -8
                    sd   a1, (sp)

                    li   a1, {offset}
                    add  a0, a0, a1

                    ld   a1, (sp)
                    addi sp, sp,  8

                    sd   a1, (a0)
                    ret
                ",
                offset = const Self::MTIME_OFFSET,
            )
        }
    }

    #[naked]
    pub extern "C" fn read_mtimecmp_naked(&self, hart_idx: usize) -> u64 {
        unsafe {
            naked_asm!(
                "   slli a1, a1, 3
                    add  a0, a0, a1

                    li   a1, {offset}
                    add  a0, a0, a1

                    ld   a0, (a0)
                    ret
                ",
                offset = const Self::MTIMER_OFFSET,
            )
        }
    }

    #[naked]
    pub extern "C" fn write_mtimecmp_naked(&self, hart_idx: usize, val: u64) {
        unsafe {
            naked_asm!(
                "   slli a1, a1, 3
                    add  a0, a0, a1

                    li   a1, {offset}
                    add  a0, a0, a1

                    sd   a2, (a0)
                    ret
                ",
                offset = const Self::MTIMER_OFFSET,
            )
        }
    }

    #[naked]
    pub extern "C" fn read_msip_naked(&self, hart_idx: usize) -> bool {
        unsafe {
            naked_asm!(
                "   slli a1, a1, 2
                    add  a0, a0, a1
                    lw   a0, (a0)
                    ret
                ",
            )
        }
    }

    #[naked]
    pub extern "C" fn set_msip_naked(&self, hart_idx: usize) {
        unsafe {
            naked_asm!(
                "   slli a1, a1, 2
                    add  a0, a0, a1
                    addi a1, zero, 1
                    sw   a1, (a0)
                    ret
                ",
            )
        }
    }

    #[naked]
    pub extern "C" fn clear_msip_naked(&self, hart_idx: usize) {
        unsafe {
            naked_asm!(
                "   slli a1, a1, 2
                    add  a0, a0, a1
                    sw   zero, (a0)
                    ret
                ",
            )
        }
    }
}

#[test]
fn test() {
    assert_eq!(core::mem::size_of::<MSWI>(), 0x3ffc);
    assert_eq!(core::mem::size_of::<SSWI>(), 0x3ffc);
    assert_eq!(core::mem::size_of::<MTIMER>(), 0x7ff8);
    assert_eq!(core::mem::size_of::<SifiveClint>(), 0xc000);
}
