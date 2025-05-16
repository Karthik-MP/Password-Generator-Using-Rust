use std::cell::Cell;

/// A simple wrapper around `u128` representing a numerical base (radix),
/// used for converting numbers into custom base encodings (e.g., printable ASCII).
///
/// Internally uses `Cell<u128>` for interior mutability, although the current API
/// only exposes read access.
///
/// # Examples
///
/// ```rust
/// let radix = Radix::new(95); // Printable ASCII
/// assert_eq!(radix.get(), 95);
/// ```
pub(crate) struct Radix(Cell<u128>);

/// Creates a new `Radix` instance with the given value.
///
/// # Parameters
/// - `value`: The radix base (e.g., 10, 16, 95).
///
/// # Returns
/// A `Radix` instance containing the specified base.
///
/// # Example
///
/// ```rust
/// let r = Radix::new(36);
/// ```
impl Radix {
    // Constructor to create a Radix instance
    pub fn new(value: u128) -> Self {
        Self(Cell::new(value))
    }

    // Getter to retrieve the value
    pub fn get(&self) -> u128 {
        self.0.get()
    }
}
