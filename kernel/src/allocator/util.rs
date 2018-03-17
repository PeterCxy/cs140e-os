/// Align `addr` downwards to the nearest multiple of `align`.
///
/// The returned usize is always <= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    assert_power_of_two(align);
    addr - (addr % align)
}

/// Align `addr` upwards to the nearest multiple of `align`.
///
/// The returned `usize` is always >= `addr.`
///
/// # Panics
///
/// Panics if `align` is not a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    assert_power_of_two(align);
    let offset = addr % align;
    if offset != 0 {
        addr + (align - offset)
    } else {
        addr
    }
}

#[inline(always)]
fn assert_power_of_two(num: usize) {
    if !is_power_of_two(num) {
        panic!("Alignment not a power of 2");
    }
}

#[inline(always)]
pub fn is_power_of_two(num: usize) -> bool {
    (num != 0) && (num & (num - 1) == 0)
}

// Simple and naive log2_ceil implementation
pub fn log2_ceil(num: usize) -> usize {
    let mut x = 1;
    let mut i = 0;
    while x < num {
        x = x << 1;
        i += 1;
    }
    return i;
}