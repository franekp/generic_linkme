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
        std::ptr::write_volatile(std::mem::transmute(MAYBE_VALID_RES), (self)());
    }
}

impl<A, Res, F: Fn(A) -> Res + Clone + 'static> AnyFn<(A,)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_A));
        std::ptr::write_volatile(std::mem::transmute(MAYBE_VALID_RES), (self)(a));
    }
}

impl<A, B, Res, F: Fn(A, B) -> Res> AnyFn<(A, B)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_A));
        let b = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_B));
        std::ptr::write_volatile(std::mem::transmute(MAYBE_VALID_RES), (self)(a, b));
    }
}

impl<A, B, C, Res, F: Fn(A, B, C) -> Res> AnyFn<(A, B, C)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_A));
        let b = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_B));
        let c = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_C));
        std::ptr::write_volatile(std::mem::transmute(MAYBE_VALID_RES), (self)(a, b, c));
    }
}

impl<A, B, C, D, Res, F: Fn(A, B, C, D) -> Res> AnyFn<(A, B, C, D)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_A));
        let b = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_B));
        let c = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_C));
        let d = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_D));
        std::ptr::write_volatile(std::mem::transmute(MAYBE_VALID_RES), (self)(a, b, c, d));
    }
}

impl<A, B, C, D, E, Res, F: Fn(A, B, C, D, E) -> Res> AnyFn<(A, B, C, D, E)> for F {
    unsafe fn dry_run(&self) {
        let a = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_A));
        let b = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_B));
        let c = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_C));
        let d = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_D));
        let e = std::ptr::read_volatile(std::mem::transmute(MAYBE_VALID_E));
        std::ptr::write_volatile(std::mem::transmute(MAYBE_VALID_RES), (self)(a, b, c, d, e));
    }
}
