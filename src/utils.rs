pub fn extract_next_sixteen_bits_from_buffer(buffer: &[u8]) -> (u16, &[u8]) {
    let (n, rest) = buffer.split_at(2);
    (concat_two_u8s(n[0], n[1]), rest)
}

pub fn concat_two_u8s(a: u8, b: u8) -> u16 {
    (a as u16) << 8 | b as u16
}

pub fn extract_next_thirty_two_bits_from_buffer(buffer: &[u8]) -> (u32, &[u8]) {
    let (n, rest) = buffer.split_at(4);
    (concat_four_u8s(n[0], n[1], n[2], n[3]), rest)
}

pub fn concat_four_u8s(a: u8, b: u8, c: u8, d: u8) -> u32 {
    (a as u32) << 24 | (b as u32) << 16 | (c as u32) << 8 | d as u32
}

pub fn split_two_bytes(n: u16) -> (u8, u8) {
    ((n >> 8) as u8, n as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_extract_next_sixteen_bits_from_buffer() {
        let buffer = &[0b01110101, 0b10110100, 0b10011101, 0b00011101];
        let (result, rest) = extract_next_sixteen_bits_from_buffer(buffer);
        assert_eq!(0b0111010110110100, result);
        assert_eq!(&[0b10011101, 0b00011101], rest);
    }

    #[test]
    fn test_concat_two_u8s() {
        let (a, b) = (0b01110101, 0b10110100);
        let expected = 0b0111010110110100; // manual concatenation of the numbers above

        assert_eq!(expected, concat_two_u8s(a, b));
    }

    #[test]
    fn test_extract_next_thirty_two_bits_from_buffer() {
        let buffer = &[
            0b01110101,
            0b10110100,
            0b10011101,
            0b00011101,
            0b11111101,
            0b000100001,
        ];
        let (result, rest) = extract_next_thirty_two_bits_from_buffer(buffer);
        assert_eq!(0b01110101101101001001110100011101, result);
        assert_eq!(&[0b11111101, 0b000100001], rest);
    }

    #[test]
    fn test_concat_four_u8s() {
        let (a, b, c, d) = (0b01110101, 0b10110100, 0b10011101, 0b00011101);
        let expected = 0b01110101101101001001110100011101; // manual concatenation of the numbers above

        assert_eq!(expected, concat_four_u8s(a, b, c, d));
    }

    #[test]
    fn test_split_two_bytes() {
        let n = 0b0101101011010111;
        let expected = (0b01011010, 0b11010111);

        assert_eq!(expected, split_two_bytes(n));
    }
}
