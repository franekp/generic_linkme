use core::mem;
use core::ops::Deref;
use core::slice;
use once_cell::sync::OnceCell;
use capstone::prelude::*;

use crate::__private::Slice;

pub struct DistributedFnSlice<T: ?Sized + Slice + 'static> {
    name: &'static str,
    section_start: *const u8,
    section_stop: *const u8,
    dupcheck_start: *const usize,
    dupcheck_stop: *const usize,
    slice: OnceCell<&'static T>,
}

unsafe impl<T: ?Sized + Slice> Send for DistributedFnSlice<T> {}

unsafe impl<T: ?Sized + Slice> Sync for DistributedFnSlice<T> {}

impl<T: ?Sized + Slice> Clone for DistributedFnSlice<T> {
    fn clone(&self) -> Self {
        DistributedFnSlice {
            name: self.name,
            section_start: self.section_start,
            section_stop: self.section_stop,
            dupcheck_start: self.dupcheck_start,
            dupcheck_stop: self.dupcheck_stop,
            slice: self.slice.clone(),
        }
    }
}

impl<T> DistributedFnSlice<[T]> {
    #[doc(hidden)]
    #[cfg(any(
        target_os = "none",
        target_os = "linux",
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "illumos",
        target_os = "freebsd"
    ))]
    pub const unsafe fn private_new(
        name: &'static str,
        section_start: *const u8,
        section_stop: *const u8,
        dupcheck_start: *const usize,
        dupcheck_stop: *const usize,
    ) -> Self {
        DistributedFnSlice {
            name,
            section_start,
            section_stop,
            dupcheck_start,
            dupcheck_stop,
            slice: OnceCell::new(),
        }
    }

    #[doc(hidden)]
    #[cfg(target_os = "windows")]
    pub const unsafe fn private_new(
        name: &'static str,
        section_start: *const [u8; 0],
        section_stop: *const [u8; 0],
        dupcheck_start: *const (),
        dupcheck_stop: *const (),
    ) -> Self {
        DistributedFnSlice {
            name,
            section_start: section_start as *const u8,
            section_stop: section_stop as *const u8,
            dupcheck_start: dupcheck_start as *const usize,
            dupcheck_stop: dupcheck_stop as *const usize,
            slice: OnceCell::new(),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn private_typecheck(&self, element: T) {
        mem::forget(element);
    }
}

impl<T> DistributedFnSlice<[T]> {
    fn get_code(&self) -> &'static [u8] {
        let len = self.section_stop as usize - self.section_start as usize;
        unsafe { slice::from_raw_parts(self.section_start, len) }
    }

    pub fn static_slice(&self) -> &'static [T] {
        if self.dupcheck_start.wrapping_add(1) < self.dupcheck_stop {
            panic!("duplicate #[distributed_slice] with name \"{}\"", self.name);
        }

        match self.slice.get() {
            Some(slice) => slice,
            None => {
                let res: &'static [T] = Box::leak(extract_function_pointers::<T>(self.get_code()).into_boxed_slice());
                match self.slice.set(res) {
                    Ok(()) => res,
                    Err(res) => {
                        unsafe {
                            std::mem::drop(Box::<[T]>::from_raw(
                                std::mem::transmute::<*const [T], *mut [T]>(res as *const [T])
                            ))
                        }
                        self.slice.get().unwrap()
                    },
                }
            }
        }
    }
}

impl<T: 'static> Deref for DistributedFnSlice<[T]> {
    type Target = [T];
    fn deref(&self) -> &'static Self::Target {
        self.static_slice()
    }
}

impl<T: 'static> IntoIterator for &DistributedFnSlice<[T]> {
    type Item = &'static T;
    type IntoIter = slice::Iter<'static, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.static_slice().iter()
    }
}

fn extract_function_pointers<T>(code: &[u8]) -> Vec<T> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");
    let addr = code.as_ptr() as usize;
    let insns = cs.disasm_all(code, addr as u64)
        .expect("Failed to disassemble");
    let mut v = Vec::new();
    for i in insns.as_ref() {
        let Some(name) = cs.insn_name(i.id()) else { continue };
        if name == "call" || name == "jmp" {
            let detail: InsnDetail = cs.insn_detail(&i).expect("Failed to get insn detail");
            let arch_detail: ArchDetail = detail.arch_detail();
            let ops = arch_detail.operands();
            for op in ops {
                let op = match op {
                    arch::ArchOperand::X86Operand(op) => op,
                    _ => continue,
                };
                match op.op_type {
                    arch::x86::X86OperandType::Imm(val) => {
                        assert!(std::mem::size_of::<T>() == std::mem::size_of::<i64>());
                        let f = unsafe { std::mem::transmute_copy(&val) };
                        v.push(f);
                    },
                    _ => continue,
                }
            }
        }
    }
    v
}
