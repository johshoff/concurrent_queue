
pub fn fetch_and_add(value: &mut u64, addend: u64) -> u64 {
    let prev_value = *value;
    *value += addend;

    prev_value
}

pub fn test_and_set(value: &mut u64) {
    // TODO: Is this the desired poly-fill?
    *value = 0xffffffffffffffff; // 64 bits
}

pub fn compare_and_swap(current: &mut u64, expected: u64, new_value: u64) -> bool { // TODO: return Result to pass back values?
    if *current == expected {
        *current = new_value;
        true
    } else {
        false
    }
}

pub fn compare_and_swap_2(current: &mut [u64; 2], expected: &[u64; 2], new_value: &[u64; 2]) -> bool { // TODO: return Result to pass back values?
    if current[0] == expected[0] &&
       current[1] == expected[1] {
        current[0] = new_value[0];
        current[1] = new_value[1];
        true
    } else {
        false
    }
}

#[test]
fn test_fetch_and_add() {
    let mut x = 5;
    assert_eq!(fetch_and_add(&mut x, 1), 5);
    assert_eq!(fetch_and_add(&mut x, 5), 6);
    assert_eq!(x, 11);
}

