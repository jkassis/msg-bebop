mod lib_tests;
mod trx_tests;

pub mod context;
mod context_tests;
pub mod courier;
pub mod db;
pub mod db_sled;
mod db_sled_tests;
pub mod error;
pub mod expiration;
pub mod idempotency;
pub mod observability;
pub mod rustie {
    pub mod msg {
        pub mod msg;
        pub mod tx_sync;
        pub mod txrx;
    }
    pub mod trx {
        pub mod msg;
        pub mod tx_sync;
        pub mod txrx;
    }
}
pub mod pact;
pub mod receipt;

pub use context::*;
pub use courier::*;
pub use db::*;
pub use db_sled::*;
pub use error::*;
pub use expiration::*;
pub use idempotency::*;
pub use observability::*;
pub use pact::*;
pub use rustie::msg::msg::*;
pub use rustie::msg::txrx::*;
pub use rustie::trx as trx;

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
