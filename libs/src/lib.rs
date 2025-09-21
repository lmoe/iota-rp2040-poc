#![feature(type_alias_impl_trait)]
#![cfg_attr(not(feature = "std"), no_std)]
#![feature(trivial_bounds)]
#[cfg(not(feature = "std"))]
use core::prelude::v1::*;

extern crate alloc;
extern crate core;
#[cfg(feature = "std")]
extern crate std;

pub mod crypto;
pub mod encoding;
pub mod gas_station_client;
pub mod json_client;
pub mod transaction_types;
