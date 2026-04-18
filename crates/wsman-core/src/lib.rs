#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod error;
pub mod schema;
pub mod selector;

pub use error::WsmanError;
