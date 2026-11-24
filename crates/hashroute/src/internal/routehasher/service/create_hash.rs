use nanoid::nanoid;

/// Alphabet for short codes: 62 URL-safe alphanumeric characters.
const ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

/// Generates a random short code of `size` characters (at least 1).
pub fn generate_short_code(size: usize) -> String {
    nanoid!(size.max(1), &ALPHABET)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_single_char() {
        assert_eq!(1, generate_short_code(1).len());
    }

    #[test]
    fn test_zero_is_clamped_to_one() {
        assert_eq!(1, generate_short_code(0).len());
    }

    #[test]
    fn test_10_char() {
        assert_eq!(10, generate_short_code(10).len());
    }

    #[test]
    fn test_only_alphanumeric() {
        assert!(generate_short_code(64).chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
