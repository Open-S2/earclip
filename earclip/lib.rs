#![no_std]
// #![deny(missing_docs)]
//! The `earclip` Rust crate... TODO

// https://github.com/MIERUNE/earcut-rs - not quite correct, but a good place to compare performance against

/// Add two usize numbers into one
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(1, 2);
        let result2 = add(1, 1);

        assert_eq!(result, 3);
        assert_eq!(result2, 2);
    }
}
