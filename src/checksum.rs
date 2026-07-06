pub fn crc16(data: &[u8]) -> u16 {
    let mut crc = 0x6d2bu16;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

pub fn folded_sum(data: &[u8]) -> u32 {
    let mut acc = 0x9e37_79b9u32;
    for (i, &b) in data.iter().enumerate() {
        acc ^= (b as u32) << ((i % 4) * 8);
        acc = acc.rotate_left(5).wrapping_mul(0x45d9_f3b);
    }
    acc
}

pub fn looks_like_text(data: &[u8]) -> bool {
    let printable = data
        .iter()
        .filter(|&&b| b == b'\n' || b == b'\r' || b == b'\t' || (0x20..=0x7e).contains(&b))
        .count();
    printable * 4 >= data.len().saturating_mul(3)
}
