//! Module containing L1 Attributes types (aka the L1 block info transaction).
//!
//! # Hardfork Inheritance
//!
//! The hardforks build upon each other in the following chronological order:
//! - **Ecotone** (base): Introduces the common L1 block fields shared across later hardforks
//! - **Isthmus**: Extends Ecotone with operator fee fields (operator_fee_scalar, operator_fee_constant)
//! - **Jovian**: Extends Isthmus with DA footprint field (da_footprint_gas_scalar)
//! - **Interop**: Shares the same structure as Ecotone but with a different selector (not yet active)
//!
//! Each hardfork reuses encoding/decoding logic from its parent hardfork to reduce code
//! duplication and maintain consistency.

mod variant;
pub use variant::L1BlockInfoTx;

mod isthmus;
pub use isthmus::L1BlockInfoIsthmus;

mod bedrock;
pub use bedrock::L1BlockInfoBedrock;

mod ecotone;
pub use ecotone::L1BlockInfoEcotone;

mod jovian;
pub use jovian::L1BlockInfoJovian;

// Interop is not yet active, but is prepared for future use
mod interop;
#[allow(unused_imports)]
pub(crate) use interop::L1BlockInfoInterop;

mod errors;
pub use errors::{BlockInfoError, DecodeError};

mod common;
pub(crate) use common::CommonL1BlockFields;
