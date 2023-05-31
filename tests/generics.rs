use std::any::{Any, type_name};
use std::rc::Rc;
use generic_linkme::{distributed_fn_slice, link};

mod declarations {
    use super::*;

    #[distributed_fn_slice]
    pub static BY_RET_VAL: [fn() -> String] = [..];

    #[distributed_fn_slice]
    pub static BY_VEC_PUSH: [for<'a> fn(&'a mut Vec<&'static str>) -> &'a mut &'static str] = [..];

    #[distributed_fn_slice]
    pub static BY_OWNED_ARRAY: [fn([String; 100], usize) -> [String; 100]] = [..];

    #[distributed_fn_slice]
    pub static BY_MUTABLE_CONT: [fn(&mut dyn FnMut(&'static str))] = [..];

    #[distributed_fn_slice]
    pub static BY_IMMUTABLE_CONT: [fn(&dyn Fn(String) -> Rc<dyn Any>) -> Rc<dyn Any>] = [..];
}

mod elements {
    use super::*;

    #[distributed_fn_slice(declarations::BY_RET_VAL)]
    fn by_ret_val_1<T: ?Sized>() -> String {
        type_name::<T>().to_string()
    }

    #[distributed_fn_slice(declarations::BY_RET_VAL)]
    fn by_ret_val_2<T, U>() -> String {
        format!("{}, {}", type_name::<T>(), type_name::<U>())
    }

    #[distributed_fn_slice(declarations::BY_RET_VAL)]
    fn by_ret_val_fn_trait<F: Fn(u32, u32) -> u32>() -> String {
        type_name::<F>().to_string()
    }

    #[distributed_fn_slice(declarations::BY_VEC_PUSH)]
    fn by_vec_push_1<'a, T: ?Sized>(v: &'a mut Vec<&'static str>) -> &'a mut &'static str {
        let res = type_name::<T>();
        v.push(res);
        v.last_mut().unwrap()
    }

    #[distributed_fn_slice(declarations::BY_VEC_PUSH)]
    fn by_vec_push_2<'a, T, U>(v: &'a mut Vec<&'static str>) -> &'a mut &'static str {
        let res = Box::leak(format!("{}, {}", type_name::<T>(), type_name::<U>()).into_boxed_str());
        v.push(res);
        v.last_mut().unwrap()
    }

    #[distributed_fn_slice(declarations::BY_VEC_PUSH)]
    fn by_vec_push_fn_trait<'a, F>(v: &'a mut Vec<&'static str>) -> &'a mut &'static str
    where F: Fn(u32, u32) -> u32 {
        let res = type_name::<F>();
        v.push(res);
        v.last_mut().unwrap()
    }

    #[distributed_fn_slice(declarations::BY_OWNED_ARRAY)]
    fn by_owned_array_1<T: ?Sized>(mut a: [String; 100], i: usize) -> [String; 100] {
        a[i] = type_name::<T>().to_string();
        a
    }

    #[distributed_fn_slice(declarations::BY_OWNED_ARRAY)]
    fn by_owned_array_2<T, U>(mut a: [String; 100], i: usize) -> [String; 100] {
        a[i] = format!("{}, {}", type_name::<T>(), type_name::<U>());
        a
    }

    #[distributed_fn_slice(declarations::BY_OWNED_ARRAY)]
    fn by_owned_array_fn_trait<F: Fn(u32, u32) -> u32>(mut a: [String; 100], i: usize) -> [String; 100] {
        a[i] = type_name::<F>().to_string();
        a
    }

    #[distributed_fn_slice(declarations::BY_MUTABLE_CONT)]
    fn by_mutable_cont_1<T: ?Sized>(f: &mut dyn FnMut(&'static str)) {
        f(type_name::<T>())
    }

    #[distributed_fn_slice(declarations::BY_MUTABLE_CONT)]
    fn by_mutable_cont_2<T, U>(f: &mut dyn FnMut(&'static str)) {
        f(Box::leak(format!("{}, {}", type_name::<T>(), type_name::<U>()).into_boxed_str()))
    }

    #[distributed_fn_slice(declarations::BY_MUTABLE_CONT)]
    fn by_mutable_cont_fn_trait<F>(f: &mut dyn FnMut(&'static str)) where F: Fn(u32, u32) -> u32 {
        f(type_name::<F>())
    }

    #[distributed_fn_slice(declarations::BY_IMMUTABLE_CONT)]
    fn by_immutable_cont_1<T: ?Sized>(f: &dyn Fn(String) -> Rc<dyn Any>) -> Rc<dyn Any> {
        f(type_name::<T>().to_string())
    }

    #[distributed_fn_slice(declarations::BY_IMMUTABLE_CONT)]
    fn by_immutable_cont_2<T, U>(f: &dyn Fn(String) -> Rc<dyn Any>) -> Rc<dyn Any> {
        f(format!("{}, {}", type_name::<T>(), type_name::<U>()))
    }

    #[distributed_fn_slice(declarations::BY_IMMUTABLE_CONT)]
    fn by_immutable_cont_fn_trait<F: Fn(u32, u32) -> u32>(f: &dyn Fn(String) -> Rc<dyn Any>) -> Rc<dyn Any> {
        f(type_name::<F>().to_string())
    }

    pub fn link_elements() {
        link(by_ret_val_1::<str>);
        link(by_ret_val_2::<u32, u64>);
        link(by_ret_val_fn_trait::<fn(u32, u32) -> u32>);
        link(by_vec_push_1::<str>);
        link(by_vec_push_2::<u32, u64>);
        link(by_vec_push_fn_trait::<fn(u32, u32) -> u32>);
        link(by_owned_array_1::<str>);
        link(by_owned_array_2::<u32, u64>);
        link(by_owned_array_fn_trait::<fn(u32, u32) -> u32>);
        link(by_mutable_cont_1::<str>);
        link(by_mutable_cont_2::<u32, u64>);
        link(by_mutable_cont_fn_trait::<fn(u32, u32) -> u32>);
        link(by_immutable_cont_1::<str>);
        link(by_immutable_cont_2::<u32, u64>);
        link(by_immutable_cont_fn_trait::<fn(u32, u32) -> u32>);
    }
}

mod results {
    use super::*;
    use super::declarations::*;

    pub fn by_ret_val() -> Vec<String> {
        let mut v = Vec::new();
        for f in &BY_RET_VAL {
            v.push(f());
        }
        v
    }

    pub fn by_vec_push() -> Vec<String> {
        let mut v = Vec::new();
        for f in &BY_VEC_PUSH {
            f(&mut v);
        }
        v.into_iter().map(|s| s.to_string()).collect()
    }

    pub fn by_owned_array() -> Vec<String> {
        const EMPTY: String = String::new();
        let mut arr = [EMPTY; 100];
        for (i, f) in BY_OWNED_ARRAY.iter().enumerate() {
            arr = f(arr, i);
        }
        arr[..BY_OWNED_ARRAY.len()].to_vec()
    }

    pub fn by_mutable_cont() -> Vec<String> {
        let mut v = Vec::new();
        for f in &BY_MUTABLE_CONT {
            f(&mut |s| v.push(s.to_string()));
        }
        v.into_iter().map(|s| s.to_string()).collect()
    }

    pub fn by_immutable_cont() -> Vec<String> {
        let mut v = Vec::new();
        for f in &BY_IMMUTABLE_CONT {
            v.push(f(&|s| Rc::new(s)));
        }
        v.into_iter().map(|s| (&*s.downcast::<String>().unwrap()).clone()).collect()
    }

    pub fn expected() -> Vec<String> {
        vec![
            type_name::<str>().to_string(),
            format!("{}, {}", type_name::<u32>(), type_name::<u64>()),
            type_name::<fn(u32, u32) -> u32>().to_string(),
        ]
    }
}

#[test]
fn by_ret_val() {
    let mut v = results::by_ret_val();
    let mut e = results::expected();
    v.sort(); e.sort();
    assert_eq!(v, e);
    elements::link_elements();
}

#[test]
fn by_vec_push() {
    let mut v = results::by_vec_push();
    let mut e = results::expected();
    v.sort(); e.sort();
    assert_eq!(v, e);
    elements::link_elements();
}

#[test]
fn by_owned_array() {
    let mut v = results::by_owned_array();
    let mut e = results::expected();
    v.sort(); e.sort();
    assert_eq!(v, e);
    elements::link_elements();
}

#[test]
fn by_mutable_cont() {
    let mut v = results::by_mutable_cont();
    let mut e = results::expected();
    v.sort(); e.sort();
    assert_eq!(v, e);
    elements::link_elements();
}

#[test]
fn by_immutable_cont() {
    let mut v = results::by_immutable_cont();
    let mut e = results::expected();
    v.sort(); e.sort();
    assert_eq!(v, e);
    elements::link_elements();
}
