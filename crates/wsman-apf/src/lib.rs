#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod error;

pub use error::{ApfError, HeciError};
