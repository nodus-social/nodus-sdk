//! The verbs that change a note's interaction state.
//!
//! Favourites and reactions are two different things on Misskey, and neither is a
//! flag the timeline can fill in: favouriting answers with the note, reacting answers
//! with an empty object. The state a caller renders after a tap therefore has to be
//! read back deliberately, and these round trips live here so `lib.rs` stays about
//! the read surface.

use nodus_client_core::Result;
use nodus_client_core::rest::Transport;
use serde_json::{Value, json};

use crate::MisskeyInstance;

/// The reactions on one note, keyed the way Misskey stores them.
///
/// A key is either a Unicode reaction (`"❤️"`) or a custom emoji shortcode
/// (`":blob:"`). Both are kept verbatim because the same spelling is what a caller
/// has to send back to [`MisskeyInstance::react`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reactions {
    counts: std::collections::BTreeMap<String, u64>,
}

impl Reactions {
    /// Read the tally out of a note payload.
    ///
    /// A note without a `reactions` field has none; an entry whose count is not a
    /// number is dropped rather than failing the whole note.
    #[must_use]
    pub fn from_note(note: &Value) -> Self {
        let counts = note
            .get("reactions")
            .and_then(Value::as_object)
            .map(|map| {
                map.iter()
                    .filter_map(|(emoji, count)| count.as_u64().map(|count| (emoji.clone(), count)))
                    .collect()
            })
            .unwrap_or_default();
        Self { counts }
    }

    /// How many accounts picked `emoji`, or 0 when nobody did.
    #[must_use]
    pub fn count(&self, emoji: &str) -> u64 {
        self.counts.get(emoji).copied().unwrap_or_default()
    }

    /// Whether anyone picked `emoji`.
    #[must_use]
    pub fn contains(&self, emoji: &str) -> bool {
        self.counts.contains_key(emoji)
    }

    /// Every reaction with at least one account, in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, u64)> + '_ {
        self.counts
            .iter()
            .map(|(emoji, count)| (emoji.as_str(), *count))
    }

    /// Every reaction summed.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Whether the note has no reactions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }
}

impl<T: Transport> MisskeyInstance<T> {
    /// Favourites a note and reports whether it is favourited afterwards.
    ///
    /// Misskey's favourite is a private star, separate from reactions. The verb
    /// answers with the whole note, so `isFavorited` is the authoritative answer when
    /// the instance sends it; when it does not (the nodus backend omits it), the
    /// verb's own outcome is the only state there is to report.
    pub async fn favourite(&self, note_id: &str) -> Result<bool> {
        let note = self
            .call("/api/notes/favorites/create", json!({ "noteId": note_id }))
            .await?;
        Ok(note
            .get("isFavorited")
            .and_then(Value::as_bool)
            .unwrap_or(true))
    }

    /// Removes the account's favourite and reports the state afterwards.
    ///
    /// The wire shape is [`favourite`](Self::favourite)'s with the delete verb; the
    /// same rule decides what the returned flag is.
    pub async fn unfavourite(&self, note_id: &str) -> Result<bool> {
        let note = self
            .call("/api/notes/favorites/delete", json!({ "noteId": note_id }))
            .await?;
        Ok(note
            .get("isFavorited")
            .and_then(Value::as_bool)
            .unwrap_or(false))
    }

    /// Adds the account's reaction and returns the note's new tally.
    ///
    /// `notes/reactions/create` answers with an empty object, so the tally has to be
    /// read from the note again; a locally incremented guess would drift from every
    /// other viewer's count.
    pub async fn react(&self, note_id: &str, emoji: &str) -> Result<Reactions> {
        self.call(
            "/api/notes/reactions/create",
            json!({ "noteId": note_id, "reaction": emoji }),
        )
        .await?;
        self.reactions(note_id).await
    }

    /// Removes the account's reaction and returns the note's new tally.
    ///
    /// Misskey deletes whichever reaction the account placed on the note, so `emoji`
    /// is not part of the request; it is accepted to mirror [`react`](Self::react),
    /// and the returned tally is what actually changed.
    pub async fn unreact(&self, note_id: &str, _emoji: &str) -> Result<Reactions> {
        self.call("/api/notes/reactions/delete", json!({ "noteId": note_id }))
            .await?;
        self.reactions(note_id).await
    }

    /// Read a note's reaction tally without converting it into a status.
    ///
    /// Public because a caller that holds only a note id — a view drawing the chips on a
    /// note it was addressed by, not one it read out of a timeline — has no raw payload to
    /// derive [`Reactions`] from, and a [`Status`](nodus_client_core::model::Status) has no
    /// field that could carry the tally this far.
    pub async fn reactions(&self, note_id: &str) -> Result<Reactions> {
        let note = self
            .call("/api/notes/show", json!({ "noteId": note_id }))
            .await?;
        Ok(Reactions::from_note(&note))
    }
}

#[cfg(test)]
mod tests;
