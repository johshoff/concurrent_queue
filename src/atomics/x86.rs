
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn compare_and_swap(destination: &mut u64, expected: u64, new_value: u64) -> bool {
	let value_at_dest : u64;
    unsafe {
        asm!("LOCK CMPXCHG qword ptr [RCX], RBX"
             : "={rax}"(value_at_dest)   // output

             : "{rbx}"(new_value),
               "{rcx}"(destination),     // input
               "{rax}"(expected)

             : "{rax}", "memory"         // clobbers

             : "intel"                   // options
        );
    }

	// this information is also available through the zero flag, but it's
	// impossible (?) to use that information without doing some sort of
	// secondary compare outside of the asm! block
    value_at_dest == expected
}

#[repr(simd)] // for 16 byte alignment
#[derive(Debug)]
pub struct DoubleU64 {
    high: u64,
    low:  u64,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn compare_and_swap_2(destination: &mut DoubleU64, expected: &DoubleU64, new_value: &DoubleU64) -> bool { // TODO: return Result to pass back values?
	let value_at_dest_high : u64;
	let value_at_dest_low  : u64;

    unsafe {
        asm!("LOCK CMPXCHG16B [R8]"
             : "={rax}"(value_at_dest_high), // output
               "={rdx}"(value_at_dest_low)

             : "{rbx}"(new_value.high),      // input
               "{rcx}"(new_value.low),
               "{r8}"(destination),
               "{rax}"(expected.high)
               "{rdx}"(expected.low)

             : "{rax}", "{rdx}", "memory"    // clobbers

             : "intel"                       // options
        );
    }

	// this information is also available through the zero flag, but it's
	// impossible (?) to use that information without doing some sort of
	// secondary compare outside of the asm! block
    value_at_dest_high == expected.high && value_at_dest_low == expected.low
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn fetch_and_add(destination: &mut u64, addend: u64) -> u64 {
	let value_at_dest : u64;
    unsafe {
        asm!("LOCK XADD qword ptr [RCX], RBX"
             : "={rbx}"(value_at_dest)   // output

             : "{rbx}"(addend),          // input
               "{rcx}"(destination)

             : "{rbx}", "memory"         // clobbers

             : "intel"                   // options
        );
    }

    value_at_dest
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn test_and_set(destination: &mut u64) {
    unsafe {
        asm!("LOCK XCHG qword ptr [RCX], RBX"
             :                              // output

             : "{rbx}"(0xffffffffffffffff), // input
               "{rcx}"(destination)

             : "{rbx}", "memory"            // clobbers

             : "intel"                      // options
        );
    }
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
    fn test_compare_and_swap_2_single_thread() {
	    let mut x = DoubleU64 { high: 1, low: 2 };
	    assert!(compare_and_swap_2(&mut x, &DoubleU64 { high: 1, low: 2 }, &DoubleU64 { high: 2, low: 3 }));
	    assert_eq!(x.high, 2);
	    assert_eq!(x.low , 3);

	    assert!(!compare_and_swap_2(&mut x, &DoubleU64 { high: 1, low: 2 }, &DoubleU64 { high: 3, low: 2 }));
	    assert_eq!(x.high, 2);
	    assert_eq!(x.low , 3);
    }
}
