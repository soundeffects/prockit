mod names;
mod provides;
mod provider;
mod queries;

pub use names::Names;
use names::{NamedType, Signature};
pub use provides::Provides;
pub use provider::Provider;
pub use queries::NameQuery;
use queries::{NamedTypeQuery, SignatureQuery};
