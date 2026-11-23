//! Why the server refused a preferences write.

/// Why the server refused a preferences write.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PreferenceRefusal {
    /// The server did not recognise the key.
    #[error("unknown preference: {key}")]
    UnknownKey {
        /// The key the server named.
        key: String,
    },
    /// The server knew the key but rejected the value.
    #[error("invalid value for {key}: {value}; allowed values are {allowed}")]
    InvalidValue {
        /// The key the server named.
        key: String,
        /// The value the server rejected.
        value: String,
        /// The allowed values, as the server phrased them.
        allowed: String,
    },
}

impl PreferenceRefusal {
    /// Recognises the two `422` messages the preferences endpoint sends.
    ///
    /// Anything else returns `None`, so a caller keeps the refusal as a plain
    /// [`Error::Status`](crate::Error::Status) instead of pretending to understand it.
    #[must_use]
    pub fn parse(message: &str) -> Option<Self> {
        if let Some(key) = message.strip_prefix("unknown preference: ") {
            return Some(Self::UnknownKey {
                key: key.to_owned(),
            });
        }
        let (head, allowed) = message.rsplit_once("; allowed values are ")?;
        let head = head.strip_prefix("invalid value for ")?;
        let (key, value) = head.split_once(": ")?;
        Some(Self::InvalidValue {
            key: key.to_owned(),
            value: value.to_owned(),
            allowed: allowed.to_owned(),
        })
    }
}
