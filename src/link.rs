use rand::prelude::*;
use once_cell::sync::Lazy;

#[inline(never)]
pub extern "C" fn link<Args>(f: impl AnyFn<Args>) {
    if always_false_but_compiler_doesnt_know_that() {
        let f2 = unsafe { std::ptr::read_volatile(&f) };
        std::mem::forget(f);
        link2(&f2);
    }
}

#[inline(never)]
#[allow(improper_ctypes_definitions)]
extern "C" fn link2<Args>(f: &dyn AnyFn<Args>) {
    let f2 = unsafe { std::ptr::read_volatile(&f) };
    std::mem::forget(f);
    unsafe { f2.dry_run() }
    println!("{}", f2 as *const dyn AnyFn<Args> as *const () as usize);
}

static mut MAYBE_VALID_A: usize = 0;
static mut MAYBE_VALID_B: usize = 0;
static mut MAYBE_VALID_C: usize = 0;
static mut MAYBE_VALID_D: usize = 0;
static mut MAYBE_VALID_E: usize = 0;
static mut MAYBE_VALID_RES: usize = 0;

fn always_false_but_compiler_doesnt_know_that() -> bool {
    static RES: Lazy<bool, fn() -> bool> = Lazy::new(|| {
        let mut rng = rand::thread_rng();
        let a: u64 = rng.gen();
        let b: u64 = rng.gen();
        let c: u64 = rng.gen();
        let d: u64 = rng.gen();
        let e: u64 = rng.gen();
        let f: u64 = rng.gen();
        let g: u64 = rng.gen();
        let h: u64 = rng.gen();
        let i: u64 = rng.gen();
        unsafe {
            MAYBE_VALID_A = rng.gen();
            MAYBE_VALID_B = rng.gen();
            MAYBE_VALID_C = rng.gen();
            MAYBE_VALID_D = rng.gen();
            MAYBE_VALID_E = rng.gen();
            MAYBE_VALID_RES = rng.gen();
            #[cfg(target_os = "windows")] {
                MAYBE_VALID_A = Box::leak(Box::new(rng.gen::<[u64; 32]>())) as *mut u64 as usize;
                MAYBE_VALID_B = Box::leak(Box::new(rng.gen::<[u64; 32]>())) as *mut u64 as usize;
                MAYBE_VALID_C = Box::leak(Box::new(rng.gen::<[u64; 32]>())) as *mut u64 as usize;
                MAYBE_VALID_D = Box::leak(Box::new(rng.gen::<[u64; 32]>())) as *mut u64 as usize;
                MAYBE_VALID_E = Box::leak(Box::new(rng.gen::<[u64; 32]>())) as *mut u64 as usize;
                MAYBE_VALID_RES = Box::leak(Box::new(rng.gen::<[u64; 32]>())) as *mut u64 as usize;
            }
        }
        return a == b
            && b == c
            && c == d
            && d == e
            && e == f
            && f == g
            && g == h
            && h == i
            && i == 42
    });
    *RES
}

pub trait AnyFn<Args> {
    unsafe fn dry_run(&self);
}

impl<Res, F: Fn() -> Res> AnyFn<()> for F {
    unsafe fn dry_run(&self) {
        std::ptr::write_volatile(MAYBE_VALID_RES as *mut Res, (self)());
    }
}

impl<A, Res, F: Fn(A) -> Res + Clone + 'static> AnyFn<(A,)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(MAYBE_VALID_A as *const A);
        std::ptr::write_volatile(MAYBE_VALID_RES as *mut Res, (self)(a));
    }
}

impl<A, B, Res, F: Fn(A, B) -> Res> AnyFn<(A, B)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(MAYBE_VALID_A as *const A);
        let b = std::ptr::read_volatile(MAYBE_VALID_B as *const B);
        std::ptr::write_volatile(MAYBE_VALID_RES as *mut Res, (self)(a, b));
    }
}

impl<A, B, C, Res, F: Fn(A, B, C) -> Res> AnyFn<(A, B, C)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(MAYBE_VALID_A as *const A);
        let b = std::ptr::read_volatile(MAYBE_VALID_B as *const B);
        let c = std::ptr::read_volatile(MAYBE_VALID_C as *const C);
        std::ptr::write_volatile(MAYBE_VALID_RES as *mut Res, (self)(a, b, c));
    }
}

impl<A, B, C, D, Res, F: Fn(A, B, C, D) -> Res> AnyFn<(A, B, C, D)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(MAYBE_VALID_A as *const A);
        let b = std::ptr::read_volatile(MAYBE_VALID_B as *const B);
        let c = std::ptr::read_volatile(MAYBE_VALID_C as *const C);
        let d = std::ptr::read_volatile(MAYBE_VALID_D as *const D);
        std::ptr::write_volatile(MAYBE_VALID_RES as *mut Res, (self)(a, b, c, d));
    }
}

impl<A, B, C, D, E, Res, F: Fn(A, B, C, D, E) -> Res> AnyFn<(A, B, C, D, E)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(MAYBE_VALID_A as *const A);
        let b = std::ptr::read_volatile(MAYBE_VALID_B as *const B);
        let c = std::ptr::read_volatile(MAYBE_VALID_C as *const C);
        let d = std::ptr::read_volatile(MAYBE_VALID_D as *const D);
        let e = std::ptr::read_volatile(MAYBE_VALID_E as *const E);
        std::ptr::write_volatile(MAYBE_VALID_RES as *mut Res, (self)(a, b, c, d, e));
    }
}
