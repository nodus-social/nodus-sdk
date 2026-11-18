//! Reading positions: where the reader stopped in each timeline.
//!
//! A marker is the only reading state the server keeps, which is what makes "continue
//! where I left off" work across devices. The version is the server's guard against a
//! stale write: a client that last synced an hour ago must not move the marker backwards
//! past what another client has already read.

use serde::{Deserialize, Serialize};

/// A timeline that carries a reading position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkerTimeline {
    Home,
    Notifications,
}

impl MarkerTimeline {
    /// Every timeline the API documents.
    pub const ALL: [Self; 2] = [Self::Home, Self::Notifications];

    /// The wire name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Notifications => "notifications",
        }
    }
}

impl core::fmt::Display for MarkerTimeline {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// How far the reader got in one timeline.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Marker {
    /// Id of the newest item the reader has seen.
    #[serde(default)]
    pub last_read_id: String,
    /// Monotonic counter; the instance keeps the higher one.
    #[serde(default)]
    pub version: u32,
    /// When the marker was last written.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// The markers an instance holds, keyed by timeline.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Markers {
    #[serde(default)]
    pub home: Option<Marker>,
    #[serde(default)]
    pub notifications: Option<Marker>,
}

impl Markers {
    /// The marker for `timeline`, when the instance has one.
    #[must_use]
    pub const fn get(&self, timeline: MarkerTimeline) -> Option<&Marker> {
        match timeline {
            MarkerTimeline::Home => self.home.as_ref(),
            MarkerTimeline::Notifications => self.notifications.as_ref(),
        }
    }
}

/// One position to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerUpdate {
    pub timeline: MarkerTimeline,
    /// Id of the newest item the reader has seen.
    pub last_read_id: String,
    /// The version this write is based on; `None` lets the instance decide.
    pub version: Option<u32>,
}

impl MarkerUpdate {
    /// A position for `timeline` at `last_read_id`, with no version attached.
    #[must_use]
    pub fn new(timeline: MarkerTimeline, last_read_id: impl Into<String>) -> Self {
        Self {
            timeline,
            last_read_id: last_read_id.into(),
            version: None,
        }
    }

    /// Attaches the version the write is based on.
    #[must_use]
    pub const fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_markers_payload_parses_each_timeline_independently() {
        let markers: Markers = serde_json::from_str(
            r#"{"home":{"last_read_id":"42","version":3},"notifications":{"last_read_id":"9"}}"#,
        )
        .expect("markers payload");
        assert_eq!(
            markers
                .get(MarkerTimeline::Home)
                .map(|marker| marker.last_read_id.as_str()),
            Some("42")
        );
        assert_eq!(
            markers
                .get(MarkerTimeline::Notifications)
                .map(|marker| marker.version),
            Some(0)
        );
    }

    #[test]
    fn an_empty_markers_object_is_legal() {
        let markers: Markers = serde_json::from_str("{}").expect("empty markers");
        assert!(markers.get(MarkerTimeline::Home).is_none());
    }
}
