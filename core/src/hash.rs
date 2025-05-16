use crate::algorithms::{
    generate_md5_hash, generate_scrypt_hash, generate_sha3_512_hash, generate_sha256_hash,
};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum HashAlgorithm {
    Md5,
    Sha256,
    Sha3_512,
    Scrypt,
}

pub fn hash_with_algorithm(password: &str, algo: &HashAlgorithm) -> Vec<u8> {
    match algo {
        HashAlgorithm::Md5 => generate_md5_hash(password.to_string()),
        HashAlgorithm::Sha256 => generate_sha256_hash(password.to_string()),
        HashAlgorithm::Sha3_512 => generate_sha3_512_hash(password.to_string()),
        HashAlgorithm::Scrypt => generate_scrypt_hash(password.to_string()),
    }
}

impl Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Md5 => write!(f, "MD5"),
            HashAlgorithm::Sha256 => write!(f, "SHA256"),
            HashAlgorithm::Sha3_512 => write!(f, "SHA3-512"),
            HashAlgorithm::Scrypt => write!(f, "Scrypt"),
        }
    }
}
