use dashmap::DashMap;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::ServerError;

/// Represents a chain in a rainbow table consisting of a start and end value.
/// Typically used in password cracking to reduce the search space.
#[derive(Clone, Debug)]
pub(crate) struct Chain {
    pub start_chain: String,
    pub end_chain: String,
}

impl Chain {
    /// Constructs a new Chain with the specified start and end values.
    ///
    /// # Arguments
    /// * `start_chain` - The starting password of the chain.
    /// * `end_chain` - The ending password of the chain.
    pub(crate) fn new(start_chain: String, end_chain: String) -> Self {
        Chain {
            start_chain,
            end_chain,
        }
    }
}

/// Represents a rainbow table which stores chains grouped by number of links.
#[derive(Debug)]
pub(crate) struct RainbowTable {
    pub num_links: DashMap<u32, Vec<Chain>>,
}

/// Holds a cracked password with its corresponding hash.
#[derive(Clone, Debug)]
pub struct CrackedPassword {
    pub hash: String,
    pub password: String,
}

impl CrackedPassword {
    /// Constructs a new CrackedPassword instance.
    ///
    /// # Arguments
    /// * `hash` - The hash that was cracked.
    /// * `password` - The corresponding plaintext password.
    pub(crate) fn new(hash: String, password: String) -> Self {
        CrackedPassword { hash, password }
    }
}

/// Internal storage for algorithm-specific data, including rainbow tables and cracked passwords.
#[derive(Debug)]
struct AlgorithmCache {
    password_len: DashMap<u32, RainbowTable>,
    cracked_passwords: DashMap<String, CrackedPassword>,
}

/// Top-level cache that holds rainbow tables and cracked passwords for multiple algorithms.
#[derive(Debug, Clone)]
pub(crate) struct Cache {
    algorithms: DashMap<String, Arc<AlgorithmCache>>,
    max_cache_size: usize,                // Max size in bytes
    current_cache_size: Arc<AtomicUsize>, // Shared mutable size counter
}

impl Cache {
    /// Creates a new, empty cache.
    pub(crate) fn new_with_size(max_cache_size: usize) -> Self {
        Cache {
            algorithms: DashMap::new(),
            max_cache_size,
            current_cache_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Inserts a new chain into the cache for a specific algorithm, password length, and number of links.
    ///
    /// # Arguments
    /// * `algorithm` - The hashing algorithm (e.g., "md5", "sha256").
    /// * `password_len` - The length of the passwords in the chain.
    /// * `num_links` - The number of links in the chain.
    /// * `chain` - The Chain object to insert.
    pub(crate) fn insert_chain(
        &self,
        algorithm: &str,
        password_len: u8,
        num_links: u32,
        chain: Chain,
    ) {
        let algo_cache = self
            .algorithms
            .entry(algorithm.to_string())
            .or_insert_with(|| {
                Arc::new(AlgorithmCache {
                    password_len: DashMap::new(),
                    cracked_passwords: DashMap::new(),
                })
            })
            .clone();

        let rainbow_table = algo_cache
            .password_len
            .entry(password_len as u32)
            .or_insert_with(|| RainbowTable {
                num_links: DashMap::new(),
            });

        rainbow_table
            .num_links
            .entry(num_links)
            .and_modify(|vec| vec.push(chain.clone()))
            .or_insert_with(|| vec![chain]);
    }

    /// Retrieves all chains for a given algorithm and password length.
    ///
    /// # Arguments
    /// * `algorithm` - The hashing algorithm to look up.
    /// * `password_len` - The length of the passwords.
    ///
    /// # Returns
    /// A map of number of links to vectors of chains or an error if no table is found.
    pub fn get_all_chains(
        &self,
        algorithm: &str,
        password_len: u8,
    ) -> Result<HashMap<u32, Vec<Chain>>, ServerError> {
        let algo_cache = self
            .algorithms
            .get(algorithm)
            .ok_or(ServerError::NoRainbowTableFound)?;

        let rainbow_table = algo_cache
            .password_len
            .get(&(password_len as u32))
            .ok_or(ServerError::NoRainbowTableFound)?;

        let mut chain_map = HashMap::new();
        for entry in rainbow_table.num_links.iter() {
            chain_map.insert(*entry.key(), entry.value().clone());
        }

        if chain_map.is_empty() {
            return Err(ServerError::NoRainbowTableFound);
        }

        Ok(chain_map)
    }

    /// Inserts a cracked password into the cache if it doesn't already exist.
    ///
    /// # Arguments
    /// * `algorithm` - The algorithm this password relates to.
    /// * `password` - The CrackedPassword object containing the hash and password.
    pub fn insert_cracked_password(&self, algorithm: &str, password: CrackedPassword) {
        let size_estimate = password.hash.len() + password.password.len(); // Rough size in bytes

        if self.current_cache_size.load(Ordering::Relaxed) + size_estimate > self.max_cache_size {
            println!("Cache full, skipping insertion for hash: {}", password.hash);
            return;
        }

        let algo_cache = self
            .algorithms
            .entry(algorithm.to_string())
            .or_insert_with(|| {
                Arc::new(AlgorithmCache {
                    password_len: DashMap::new(),
                    cracked_passwords: DashMap::new(),
                })
            })
            .clone();

        // Only insert if not already present
        let _inserted = algo_cache
            .cracked_passwords
            .entry(password.hash.clone())
            .or_insert_with(|| {
                self.current_cache_size
                    .fetch_add(size_estimate, Ordering::SeqCst);
                password
            });

        // Optional: log the updated size
        println!(
            "Cracked password inserted. Current cache size: {} / {} bytes",
            self.current_cache_size.load(Ordering::SeqCst),
            self.max_cache_size
        );
    }

    /// Retrieves a cracked password from the cache if available.
    ///
    /// # Arguments
    /// * `algorithm` - The algorithm to search in.
    /// * `hash` - The hash to look up.
    ///
    /// # Returns
    /// The CrackedPassword object or an error if not found.
    pub fn get_cracked_password(
        &self,
        algorithm: &str,
        hash: &str,
    ) -> Result<CrackedPassword, ServerError> {
        let algo_cache = self
            .algorithms
            .get(algorithm)
            .ok_or(ServerError::InvalidAlgorithm)?;

        let cracked_entry = algo_cache
            .cracked_passwords
            .get(hash)
            .ok_or(ServerError::PasswordNotFoundInCache)?;

        Ok(cracked_entry.value().clone())
    }

    /// Prints the current state of the cache including algorithms, password lengths, chains, and cracked passwords.
    ///
    /// Mainly used for debugging or inspecting cache contents.
    pub fn _print_cache(&self) {
        for algo_entry in self.algorithms.iter() {
            let algorithm = algo_entry.key();
            let algo_cache = algo_entry.value();

            println!("Algorithm: {}", algorithm);

            for pl_entry in algo_cache.password_len.iter() {
                let password_len = pl_entry.key();
                let rainbow_table = pl_entry.value();

                println!("  Password Length: {}", password_len);

                for nl_entry in rainbow_table.num_links.iter() {
                    let num_links = nl_entry.key();
                    let chains = nl_entry.value();

                    println!("    Num Links: {}", num_links);
                    for (i, chain) in chains.iter().enumerate() {
                        println!(
                            "      Chain {}: start = {}, end = {}",
                            i + 1,
                            chain.start_chain,
                            chain.end_chain
                        );
                    }
                }
            }

            println!("  Cracked Passwords:");
            for cp_entry in algo_cache.cracked_passwords.iter() {
                println!(
                    "    Hash: {}, Password: {}",
                    cp_entry.key(),
                    cp_entry.value().password
                );
            }
        }
    }
}
