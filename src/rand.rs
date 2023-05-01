
pub fn uniform(a: usize, b: usize) -> usize {
    (unsafe {js_sys::Math::random()} * (b as f64 - a as f64) + a as f64) as usize
}

pub fn bool() -> bool {
    (unsafe {js_sys::Math::random()} * 2.0) as usize == 0
}