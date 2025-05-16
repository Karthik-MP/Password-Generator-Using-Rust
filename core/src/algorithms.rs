use crate::radix_type::Radix;
use ethereum_types::{U256, U512};
use scrypt::{
    Scrypt,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use sha2::Sha256;
use sha3::{Digest, Sha3_512};

/// Generates an MD5 hash from the provided password string.
pub(crate) fn generate_md5_hash(password: String) -> Vec<u8> {
    let hash = md5::compute(&password);
    hash.to_vec()
}

pub(crate) fn generate_sha256_hash(password: String) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.finalize().to_vec()
}

pub(crate) fn generate_sha3_512_hash(password: String) -> Vec<u8> {
    let mut hasher = Sha3_512::new();
    hasher.update(password.as_bytes());
    hasher.finalize().to_vec()
}

pub(crate) fn generate_scrypt_hash(password: String) -> Vec<u8> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = match Scrypt.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => hash,
        Err(e) => return format!("Error generating scrypt hash: {e}").into_bytes(),
    };
    password_hash.to_string().into_bytes()
}

/// Reduces a hash to a printable ASCII password string.
///
/// This function performs the "reduction" step in a rainbow table hash-reduction chain.
/// It converts a hash value and a round number into a fixed-length, human-readable string
/// using the specified radix. The result is suitable for use as a plaintext input in the
/// next link of the chain.
///
/// # Parameters
///
/// - `hash`: The hash output as a byte vector (little-endian format).
/// - `round`: The current round number (used to diversify reductions across chain steps).
/// - `password_length`: The desired length of the reduced plaintext password.
/// - `radix`: A `Radix` object defining the base (e.g., 95 for printable ASCII).
///
/// # Returns
///
/// A `String` of length `password_length` made up of printable ASCII characters, derived from the hash.
///
pub(crate) fn reduction_function(
    hash: Vec<u8>,
    round: u128,
    password_length: u32,
    radix: &Radix,
) -> String {
    // let i = u128::from_le_bytes(*hash) + round;
    // println!("Hash: {:?}", hash);
    let i = U512::from_little_endian(&hash) + U256::from(round);
    let mod_by = radix.get().pow(password_length);
    let password_num: U512 = i % mod_by;
    encode(password_num, radix, password_length)
}

/// Encodes a numeric value into a printable ASCII string using a specified radix.
///
/// Converts the number into a character string by repeatedly dividing by the radix base
/// and mapping remainders to ASCII characters (by adding 32 to stay in the printable range).
///
/// # Parameters
///
/// - `num`: The `U256` number to encode.
/// - `radix`: The radix/base to use (typically 95 for printable ASCII).
/// - `length`: The desired output string length; padded with spaces if necessary.
///
/// # Returns
///
/// A `String` consisting of `length` printable ASCII characters.
///
fn encode(mut num: U512, radix: &Radix, length: u32) -> String {
    let mut s = String::new();
    let base = U512::from(radix.get());

    while num > U512::zero() {
        let (div, rem) = num.div_mod(base);
        num = div;
        let rem_u8 = rem.low_u64() as u8;
        let c = (rem_u8 + 32) as char;
        s.push(c);
    }

    while s.len() < length as usize {
        s.push(' ');
    }

    s
}
