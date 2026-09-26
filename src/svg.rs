//! SVG number formatting matching Figma's export precision

use std::fmt::Write;

/// Round to the 3 decimals Figma exports, normalising `-0` to `0`
pub fn round3(v: f64) -> f64 {
    let r = (v * 1000.0).round() / 1000.0;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}

/// Append `v` to `buf` with at most 3 decimals and no trailing zeros
pub fn write_num(buf: &mut String, v: f64) {
    let start = buf.len();
    let _ = write!(buf, "{:.3}", round3(v));
    let kept = buf[start..]
        .trim_end_matches('0')
        .trim_end_matches('.')
        .len();
    buf.truncate(start + kept);
}

/// Format `v` like [`write_num`]
pub fn num(v: f64) -> String {
    let mut s = String::new();
    write_num(&mut s, v);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_figma() {
        assert_eq!(num(1.0), "1");
        assert_eq!(num(100.0), "100");
        assert_eq!(num(-0.0001), "0");
        assert_eq!(num(12.3456), "12.346");
        assert_eq!(num(0.0005), "0.001");
        assert_eq!(num(-1.5), "-1.5");
        let mut buf = String::from("M");
        write_num(&mut buf, 6.4);
        assert_eq!(buf, "M6.4");
    }
}
