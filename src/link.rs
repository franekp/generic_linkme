use rand::prelude::*;
use once_cell::sync::Lazy;


pub fn link<Args>(f: impl AnyFn<Args>) {
    if always_false_but_compiler_doesnt_know_that() {
        unsafe { f.dry_run() }
    }
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
        unsafe {
            MAYBE_VALID_A = rng.gen();
            MAYBE_VALID_B = rng.gen();
            MAYBE_VALID_C = rng.gen();
            MAYBE_VALID_D = rng.gen();
            MAYBE_VALID_E = rng.gen();
            MAYBE_VALID_RES = rng.gen();
        }
        return a == b
            && b == 42
    });
    *RES
}

pub trait AnyFn<Args> {
    unsafe fn dry_run(&self);
}

impl<Res, F: Fn() -> Res> AnyFn<()> for F {
    unsafe fn dry_run(&self) {
        let raw_res = (self)();
        let res = std::ptr::read_volatile(&raw_res);
        std::mem::forget(raw_res);
        std::ptr::write_volatile(MAYBE_VALID_RES as *mut Res, res);
    }
}

impl<A, Res, F: Fn(A) -> Res + Clone + 'static> AnyFn<(A,)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(MAYBE_VALID_A as *const A);
        let raw_res = (self)(a);
        let res = std::ptr::read_volatile(&raw_res);
        std::mem::forget(raw_res);
        std::ptr::write_volatile(MAYBE_VALID_RES as *mut Res, res);
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
