pub fn reduce(hash: &str, password_len: usize, charset: &[u8], _ascii_offset: u8) -> String {
    let mut pwd = String::new();
    let hash_bytes = hash.as_bytes();
    let charset_len = charset.len();

    for i in 0..password_len {
        let index = hash_bytes[i % hash_bytes.len()] as usize % charset_len;
        let c = charset[index];
        pwd.push(c as char);
    }

    pwd
}
