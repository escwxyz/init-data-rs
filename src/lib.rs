#![warn(clippy::pedantic)]
// We ignore this warning because the only literals we use
// are telegram ids, which are not meant to be read
#![allow(clippy::unreadable_literal)]
mod error;
mod model;
mod parse;
mod sign;
mod third_party_validation;
mod validation;

pub use error::InitDataError;
pub use model::*;
pub use parse::parse;
pub use sign::sign;
pub use third_party_validation::validate_third_party;
pub use validation::validate;
