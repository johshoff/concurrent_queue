#![allow(mutable_transmutes)]

use std::mem;
use std::intrinsics::{atomic_cxchg, atomic_xadd};

pub fn compare_and_swap(destination: &u64, expected: u64, new_value: u64) -> bool {
    let destination : &mut u64 = unsafe { mem::transmute(destination) };
    let (value_at_dest, success) = unsafe { atomic_cxchg(destination, expected, new_value) };
    assert!((value_at_dest == expected && success) || (value_at_dest != expected && !success));

    success
}

pub fn compare_and_swap_u128(destination: &u128, expected: u128, new_value: u128) -> bool { // TODO: return Result to pass back values?
    let destination : &mut u128 = unsafe { mem::transmute(destination) };
    let (value_at_dest, success) = unsafe { atomic_cxchg(destination, expected, new_value) };
    assert!((value_at_dest == expected && success) || (value_at_dest != expected && !success));

    success
}

pub fn fetch_and_add(destination: &u64, addend: u64) -> u64 {
    let destination : &mut u64 = unsafe { mem::transmute(destination) };
    let value_at_dest = unsafe { atomic_xadd(destination, addend) };

    value_at_dest
}

pub fn test_and_set(destination: &u64) {
    //let destination : &mut u64 = unsafe { mem::transmute(destination) };
    //*destination = *destination | (1 << 63);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_fetch_and_add_single_thread() {
        let mut x = 5;
        assert_eq!(fetch_and_add(&mut x, 1), 5);
        assert_eq!(fetch_and_add(&mut x, 5), 6);
        assert_eq!(x, 11);
    }

    #[test]
    fn test_compare_and_swap_single_thread() {
	    let mut x = 42;
	    assert!(compare_and_swap(&mut x, 42, 10));

	    assert_eq!(x, 10);
	    assert!(!compare_and_swap(&mut x, 42, 11));

	    assert_eq!(x, 10);
    }

    #[test]
    fn test_compare_and_swap_u128_single_thread() {
	    let mut x  = 0x00000000000000010000000000000002u128;
	    let expect = 0x00000000000000010000000000000002u128;
	    let new    = 0x00000000000000020000000000000003u128;
	    assert!(compare_and_swap_u128(&mut x, expect, new));
	    assert_eq!(x, new);

	    // won't swap new for newer
	    let newer = 0x00000000000000030000000000000002u128;
	    assert!(!compare_and_swap_u128(&mut x, expect, newer));
	    assert_eq!(x, new);
    }

    #[test]
    fn test_test_and_set() {
        let mut x = 5;
        test_and_set(&mut x);
        assert_eq!(x, 0x8000000000000005u64);
    }
}
