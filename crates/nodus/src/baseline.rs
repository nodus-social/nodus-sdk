//! The capability answer for servers that publish no capability list.
//!
//! Nodus answers "what do you serve?" with `GET /api/v1/meta`. Mastodon and Misskey
//! have no equivalent document, so the only honest answer is a table built from their
//! documented endpoints, carried here rather than guessed at each call site.
//!
//! Each `true` names the endpoint that proves it. Each `false` means "no documented
//! endpoint this SDK can rely on", not "the server has no UI for it" — a wrong `true`
//! turns a graceful degradation into a 404 toast, so the table errs towards `false`
//! and the frontend can be corrected from one place instead of the user hitting an
//! error. Rows marked *(judgement)* had no endpoint we could verify at the time of
//! writing and are the first candidates for a correction.
//!
//! | Feature | Mastodon | Misskey |
//! |---|---|---|
//! | `admin` | `/api/v1/admin/*` (admin scope) | `/api/admin/*` (admin scope) |
//! | `moderation` | no — *(judgement)*, Mastodon's reports are admin-scoped | no *(judgement)* |
//! | `scanners` | no such concept | no such concept |
//! | `topology` | no such concept | no such concept |
//! | `protocol` | no — no capability document | no — no capability document |
//! | `remote_blocks` | no — *(judgement)*, `/api/v1/instance/domain_blocks` is read-only instance data | no *(judgement)* |
//! | `polls` | `/api/v1/polls/:id/votes` | `/api/notes/polls/vote` |
//! | `batch` | no such concept | no such concept |
//! | `appeals` | `/api/v1/appeals` | no *(judgement)* |
//! | `instance_rules` | `/api/v1/instance/rules` | `/api/meta` (`serverRules`) |
//! | `moderation_rules` | no *(judgement)* | no *(judgement)* |
//! | `media_quarantine` | no such concept | no such concept |
//! | `block_lists` | no such concept | no such concept |
//! | `moderation_stats` | no such concept | no such concept |
//! | `remote_accounts` | no — *(judgement)*, admin-scoped only | no *(judgement)* |
//! | `warnings` | no — *(judgement)*, admin-scoped only | no *(judgement)* |
//! | `escalation` | no such concept | no such concept |
//! | `federation_delivery` | no such concept | no such concept |
//! | `domain_pauses` | no such concept | no such concept |
//! | `settings_transfer` | no *(judgement)* | no *(judgement)* |
//! | `report_forwarding` | no *(judgement)* | no *(judgement)* |
//! | `account_deletion` | no *(judgement)*, no client API known | `/api/i/delete` |
//! | `drive` | no such concept | `/api/drive/files/create` |
//! | `drive_share` | no such concept | `/api/drive/files/create` with `isLink` |
//! | `drive_attach` | no such concept | `/api/notes/create` with `fileIds` |
//! | `feed_accounts` | no such concept | no such concept |
//! | `events` | no such concept | no such concept |
//! | `sync_token` | no such concept | no such concept |
//! | `favourites` | `/api/v1/favourites` | `/api/notes/favorites/create` |
//!
//! The rows a baseline adds beyond the protocol's own client API are exactly the ones
//! a screen can call without a capability probe; everything Nodus-only stays `false`.

use crate::feature::Feature;

/// The Mastodon surfaces this SDK will call without a negotiation answer.
const MASTODON: &[Feature] = &[
    Feature::Admin,
    Feature::Appeals,
    Feature::InstanceRules,
    Feature::Polls,
    Feature::Favourites,
];

/// The Misskey surfaces this SDK will call without a negotiation answer.
const MISSKEY: &[Feature] = &[
    Feature::Admin,
    Feature::AccountDeletion,
    Feature::InstanceRules,
    Feature::Polls,
    Feature::Drive,
    Feature::DriveShare,
    Feature::DriveAttach,
    Feature::Favourites,
];

/// Whether a Mastodon instance serves `feature` according to the baseline table.
pub(crate) fn mastodon_supports(feature: Feature) -> bool {
    MASTODON.contains(&feature)
}

/// Whether a Misskey instance serves `feature` according to the baseline table.
pub(crate) fn misskey_supports(feature: Feature) -> bool {
    MISSKEY.contains(&feature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mastodon_baseline_serves_polls_and_favourites() {
        assert!(mastodon_supports(Feature::Polls));
        assert!(mastodon_supports(Feature::Favourites));
        assert!(mastodon_supports(Feature::InstanceRules));
    }

    #[test]
    fn mastodon_baseline_does_not_claim_moderation_or_drive() {
        assert!(!mastodon_supports(Feature::Moderation));
        assert!(!mastodon_supports(Feature::Drive));
        assert!(!mastodon_supports(Feature::DriveAttach));
        assert!(!mastodon_supports(Feature::Batch));
    }

    #[test]
    fn misskey_baseline_serves_drive_but_not_moderation_or_batch() {
        assert!(misskey_supports(Feature::Drive));
        assert!(misskey_supports(Feature::DriveShare));
        assert!(misskey_supports(Feature::Polls));
        assert!(!misskey_supports(Feature::Moderation));
        assert!(!misskey_supports(Feature::Batch));
        assert!(!misskey_supports(Feature::FeedAccounts));
    }
}
