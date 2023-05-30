use capstone::prelude::*;
use rand::prelude::*;


#[inline(never)]
#[allow(improper_ctypes_definitions)]
extern fn inner_function<T>() -> &'static str {
    unsafe { std::ptr::read_volatile(&std::any::type_name::<T>()) }
}

#[cfg(not(target_os = "windows"))]
extern "Rust" {
    #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = "__start_slice")]
    #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = "\x01section$start$__DATA$__slice")]
    static start_slice: u8;

    #[cfg_attr(any(target_os = "none", target_os = "linux"), link_name = "__stop_slice")]
    #[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_name = "\x01section$end$__DATA$__slice")]
    static stop_slice: u8;
}

#[cfg(target_os = "windows")]
#[link_section = ".slice$a"]
static start_slice: [u8; 0] = [];

#[cfg(target_os = "windows")]
#[link_section = ".slice$c"]
static stop_slice: [u8; 0] = [];

#[used]
#[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = "slice")]
#[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_section = "__DATA,__slice,regular,no_dead_strip")]
static mut LINKME_PLEASE: [u8; 0] = [];

pub fn always_false_but_included_in_binary_1() -> bool {
    static mut IS_USED: bool = false;
    let result = unsafe { std::ptr::read_volatile::<bool>(&IS_USED as *const bool) };
    result
}

#[inline(never)]
#[cfg_attr(any(target_os = "none", target_os = "linux"), link_section = "slice")]
#[cfg_attr(any(target_os = "macos", target_os = "ios", target_os = "tvos"), link_section = "__DATA,__slice,regular,no_dead_strip")]
#[cfg_attr(target_os = "windows", link_section = ".slice$b")]
extern fn outer_function<T>() -> &'static str {
    inner_function::<T>()
}

#[cfg(not(target_os = "windows"))]
fn get_code() -> &'static [u8] {
    let len = unsafe { &stop_slice as *const u8 as usize - &start_slice as *const u8 as usize };
    let code = unsafe { std::slice::from_raw_parts(&start_slice as *const u8, len) };
    code
}

#[cfg(target_os = "windows")]
fn get_code() -> &'static [u8] {
    let len = unsafe { stop_slice as *const u8 as usize - start_slice as *const u8 as usize };
    let code = unsafe { std::slice::from_raw_parts(start_slice as *const u8, len) };
    code
}

fn disasm_code() {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");

    let code = get_code();
    println!("Code len = {}", code.len());
    let addr = &code[0] as *const u8 as usize;
    let insns = cs.disasm_all(get_code(), addr as u64)
        .expect("Failed to disassemble");
    println!("Found {} instructions", insns.len());
    for i in insns.as_ref() {
        println!();
        println!("{}", i);
        println!("{:4}ins_id: {}", "", i.id().0);
        println!("{:4}ins_name: {}", "", cs.insn_name(i.id()).expect("failed to get insn name"));

        let detail: InsnDetail = cs.insn_detail(&i).expect("Failed to get insn detail");
        let arch_detail: ArchDetail = detail.arch_detail();
        let ops = arch_detail.operands();

        println!("{:4}operands: {}", "", ops.len());
        for op in ops {
            println!("{:8}{:?}", "", op);
        }
    }
}

fn extract_fn_pointers<T>() -> Vec<T> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .expect("Failed to create Capstone object");
    let code = get_code();
    let addr = &code[0] as *const u8 as usize;
    let insns = cs.disasm_all(get_code(), addr as u64)
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

pub fn always_false_but_included_in_binary() -> bool {
    let mut rng = rand::thread_rng();
    let a: u64 = rng.gen();
    let b: u64 = rng.gen();
    return a == b && b == 100000000
}

fn main() {
    disasm_code();
    let fs = extract_fn_pointers::<extern fn() -> &'static str>();
    for f in fs {
        println!("{}", f());
    }

    if always_false_but_included_in_binary() {
        println!("{}", outer_function::<String>());
        println!("{}", outer_function::<u32>());
        println!("{}", outer_function::<bool>());
        println!("{}", outer_function::<std::cell::RefCell<bool>>());
    }
}
