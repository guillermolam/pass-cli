use rand::RngExt;

pub fn random_string(length: usize) -> String {
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".as_bytes();

    let mut res = String::new();
    while res.len() < length {
        let idx = rand::rng().random_range(0..chars.len());
        res.push(chars[idx] as char);
    }

    res
}

pub fn xor_key(key: &[u8], xor_key: u8) -> Vec<u8> {
    let mut res = Vec::with_capacity(key.len());
    for b in key {
        res.push(xor_key ^ b);
    }
    res
}

pub fn xor_key_multibyte(key: &[u8], xor_key: &[u8]) -> Vec<u8> {
    if xor_key.is_empty() {
        return key.to_vec();
    }

    let mut res = Vec::with_capacity(key.len());
    for (i, b) in key.iter().enumerate() {
        res.push(xor_key[i % xor_key.len()] ^ b);
    }
    res
}
