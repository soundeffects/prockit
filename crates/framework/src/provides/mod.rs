mod names;
mod provides;
mod queries;

pub use names::Names;
use names::{NamedType, Signature, Signatured};
pub use provides::Provides;
pub use queries::NameQuery;
use queries::NewSignatureQuery;
