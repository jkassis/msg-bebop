mod lib_tests;

pub mod context;
mod context_tests;
pub mod courier;
pub mod db;
pub mod db_sled;
mod db_sled_tests;
pub mod msg;
pub mod pact;
pub mod receipt;
pub mod txrx;
pub mod tx_sync;

pub use context::*;
pub use courier::*;
pub use db::*;
pub use db_sled::*;
pub use msg::*;
pub use pact::*;
pub use txrx::*;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
