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

// Fast log2 implementation
// Source: <https://stackoverflow.com/questions/3272424/compute-fast-log-base-2-ceiling>
const LOG2_TABLE: [usize; 6] = [
    0xFFFFFFFF00000000usize,
    0x00000000FFFF0000usize,
    0x000000000000FF00usize,
    0x00000000000000F0usize,
    0x000000000000000Cusize,
    0x0000000000000002usize
];

pub fn log2_ceil(num: usize) -> usize {
    let mut x = num;
    let mut y = if (x & (x - 1)) == 0 { 0 } else { 1 };
    let mut j = 32;
    for i in 0..6 {
        let k = if ((x & LOG2_TABLE[i]) == 0) { 0 } else { j };
        y += k;
        x = x >> k;
        j = j >> 1;
    }
    return y;
}