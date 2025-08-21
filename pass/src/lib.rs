#[macro_use]
extern crate tracing;

#[macro_use]
mod macros;

mod account;
mod cache;
mod client;
mod common;
mod constants;
mod crypto;
mod info;
mod invite;
mod item;
mod local_crypto;
mod logout;
mod pagination;
pub mod password;
mod ping;
mod share;
mod user;
mod user_keys;
mod utils;
mod vault;

#[cfg(test)]
mod test_tools;

pub use client::PassClient;
pub use item::create::login;
pub use item::find::FindItemQuery;
pub use vault::{CreateVaultArgs, UpdateVaultArgs};

pub use utils::b64_encode;
