use alloc::string::String;
use displaydoc::Display;

use crate::core::ContextError;

/// Error returned from entrypoint functions [`dispatch`][super::dispatch], [`validate`][super::validate] and
/// [`execute`][super::execute].
#[derive(Debug, Display)]
pub enum RouterError {
    /// context error: `{0}`
    ContextError(ContextError),
    /// unknown type URL `{url}`
    UnknownMessageTypeUrl { url: String },
    /// the message is malformed and cannot be decoded error: `{0}`
    MalformedMessageBytes(ibc_proto::protobuf::Error),
}

impl From<ContextError> for RouterError {
    fn from(error: ContextError) -> Self {
        Self::ContextError(error)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for RouterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::ContextError(e) => Some(e),
            Self::UnknownMessageTypeUrl { .. } => None,
            Self::MalformedMessageBytes(e) => Some(e),
        }
    }
}