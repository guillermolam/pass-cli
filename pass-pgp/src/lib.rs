#[macro_use]
extern crate tracing;

mod account;
mod pgp;

pub use account::ProtonAccountCrypto;
pub use pgp::NativePgpCrypto;
