pub fn extract_next_sixteen_bits_from_buffer(buffer: &[u8]) -> (u16, &[u8]) {
    let (n, rest) = buffer.split_at(2);
    (concat_two_u8s(n[0], n[1]), rest)
}

pub fn concat_two_u8s(a: u8, b: u8) -> u16 {
    (a as u16) << 8 | b as u16
}
