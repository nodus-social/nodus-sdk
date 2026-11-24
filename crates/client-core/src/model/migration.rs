//! Account migration: the redirect one profile publishes and the aliases that permit it.
//!
//! A move is a redirect, not a deletion: the old profile keeps answering, now with
//! `moved_to`, so a follower can choose to follow the new account. The target only accepts
//! a mover it already lists as an alias; that consent is what stops anyone from
//! redirecting somebody else's followers.

use serde::{Deserialize, Serialize};

use super::Account;

/// The compact account a `moved_to` member points at.
///
/// Nodus's account document does not embed a whole [`Account`] here. `account_move` in
/// the API builds this reference from the stored move URI, so `id`, `uri` and `url` are
/// that URI and `username`, `acct` and `domain` are parsed from it. A caller that needs
/// the rest of the profile fetches the account by `id`.
///
/// Upstream Mastodon uses its own spelling, `moved`, and puts a whole account entity
/// there instead. That entity has no `uri` or `domain` members, so those two default to
/// empty rather than failing the account decode; `id`, `url`, `username` and `acct` are
/// present in both answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovedAccount {
    /// The target's account id, which is its actor URI in nodus's answer.
    pub id: String,
    /// The target's actor URI.
    #[serde(default)]
    pub uri: String,
    /// The target's profile URL.
    #[serde(default)]
    pub url: String,
    /// The target's local username, without the domain.
    #[serde(default)]
    pub username: String,
    /// `user@host`, as the target publishes it.
    #[serde(default)]
    pub acct: String,
    /// The target's instance host.
    #[serde(default)]
    pub domain: String,
}

/// One alias exactly as `/api/v1/accounts/aliases` renders it.
///
/// `accounts` is empty when this instance has no row for the alias yet, in which case
/// `id` is the actor URI rather than an account id. That is also how a caller tells an
/// unresolved alias from a resolved one without a second lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountAlias {
    /// The target account's id, or its actor URI when this instance has no row for it.
    pub id: String,
    /// The target account documents; empty when the alias is unresolved.
    #[serde(default)]
    pub accounts: Vec<Account>,
}

/// Why the server refused a move or an alias write.
///
/// Nodus answers these refusals with a `422` and one of these sentences (an alias that
/// is not on the list is a `404`). Recognising them keeps a UI able to print the reason
/// instead of showing an unhandled status code. Anything else stays
/// [`Error::Status`](crate::Error::Status), so a caller is never told it understands a
/// refusal it does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MigrationRefusal {
    /// The signed-in account has already been redirected.
    #[error("This account has already moved")]
    AlreadyMoved,
    /// The handle or actor URI names no account this instance knows.
    #[error("This instance does not know that account")]
    UnknownAccount,
    /// Source and target are the same account.
    #[error("You cannot move to the same account")]
    SameAccount,
    /// The target has not named the mover in its aliases, so it did not consent.
    #[error(
        "The target account has not listed you in also_known_as, so it has not agreed to this move"
    )]
    TargetDidNotConsent,
    /// An account cannot list itself as its own alias.
    #[error("You cannot add your own account as an alias")]
    OwnAlias,
    /// The alias is already on the list.
    #[error("That account is already on your alias list")]
    AliasAlreadyListed,
    /// The server's alias limit has been reached.
    #[error("An account may list at most 5 aliases")]
    TooManyAliases,
    /// A concurrent write changed the list before this one landed.
    #[error("Your alias list changed; try again")]
    AliasListChanged,
    /// The alias to remove is not on the list.
    #[error("That alias is not on this account")]
    AliasNotFound,
}

impl MigrationRefusal {
    /// Recognises the sentences the migration endpoints send.
    ///
    /// The match is exact because the server's sentences carry no variable part; a
    /// refusal reworded on the server falls through to a plain status error rather than
    /// being guessed at.
    #[must_use]
    pub fn parse(message: &str) -> Option<Self> {
        match message {
            "This account has already moved" => Some(Self::AlreadyMoved),
            "This instance does not know that account" => Some(Self::UnknownAccount),
            "You cannot move to the same account" => Some(Self::SameAccount),
            "The target account has not listed you in also_known_as, so it has not agreed to this move" => {
                Some(Self::TargetDidNotConsent)
            }
            "You cannot add your own account as an alias" => Some(Self::OwnAlias),
            "That account is already on your alias list" => Some(Self::AliasAlreadyListed),
            "An account may list at most 5 aliases" => Some(Self::TooManyAliases),
            "Your alias list changed; try again" => Some(Self::AliasListChanged),
            "That alias is not on this account" => Some(Self::AliasNotFound),
            _ => None,
        }
    }

    /// Recognises Misskey's stable error codes for the same refusals.
    ///
    /// Misskey sends a code beside the message and keeps the message localised, so the
    /// code is what a client can match. `YOUR_ACCOUNT_MOVED` arrives on an alias write
    /// after the account has moved; `destinationAccountForbids` is its consent refusal.
    #[must_use]
    pub fn parse_misskey_code(code: &str) -> Option<Self> {
        match code {
            "ALREADY_MOVED" | "YOUR_ACCOUNT_MOVED" => Some(Self::AlreadyMoved),
            "NO_SUCH_USER" => Some(Self::UnknownAccount),
            "DESTINATION_ACCOUNT_FORBIDS" => Some(Self::TargetDidNotConsent),
            "FORBIDDEN_TO_SET_YOURSELF" => Some(Self::OwnAlias),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_move_target_decodes_the_six_members_nodus_sends() {
        let account: Account = serde_json::from_str(
            r#"{
                "id": "1",
                "acct": "mover",
                "moved_to": {
                    "id": "https://social.test/users/heir",
                    "uri": "https://social.test/users/heir",
                    "url": "https://social.test/users/heir",
                    "username": "heir",
                    "acct": "heir@social.test",
                    "domain": "social.test"
                }
            }"#,
        )
        .expect("account with a redirect");

        let moved = account.moved_to.expect("redirect present");
        assert_eq!(moved.id, "https://social.test/users/heir");
        assert_eq!(moved.acct, "heir@social.test");
        assert_eq!(moved.domain, "social.test");
    }

    #[test]
    fn a_plain_mastodon_moved_account_does_not_need_nodus_only_members() {
        // Upstream Mastodon calls the member `moved` and embeds a full account entity.
        // This model reads nodus's `moved_to`; the reference still decodes when `uri`
        // and `domain` are absent, which is what keeps such an answer from failing.
        let moved: MovedAccount = serde_json::from_str(
            r#"{
                "id": "2",
                "url": "https://remote.test/@heir",
                "username": "heir",
                "acct": "heir@remote.test"
            }"#,
        )
        .expect("Mastodon-shaped reference");

        assert_eq!(moved.uri, "");
        assert_eq!(moved.domain, "");
        assert_eq!(moved.acct, "heir@remote.test");
    }

    #[test]
    fn the_consent_refusal_is_recognised_by_its_sentence() {
        assert_eq!(
            MigrationRefusal::parse(
                "The target account has not listed you in also_known_as, so it has not agreed to this move"
            ),
            Some(MigrationRefusal::TargetDidNotConsent)
        );
        assert_eq!(
            MigrationRefusal::parse("This instance does not know that account"),
            Some(MigrationRefusal::UnknownAccount)
        );
        assert_eq!(MigrationRefusal::parse("some other server trouble"), None);
    }

    #[test]
    fn misskey_codes_map_to_the_same_refusals() {
        assert_eq!(
            MigrationRefusal::parse_misskey_code("DESTINATION_ACCOUNT_FORBIDS"),
            Some(MigrationRefusal::TargetDidNotConsent)
        );
        assert_eq!(
            MigrationRefusal::parse_misskey_code("NO_SUCH_USER"),
            Some(MigrationRefusal::UnknownAccount)
        );
        assert_eq!(
            MigrationRefusal::parse_misskey_code("FORBIDDEN_TO_SET_YOURSELF"),
            Some(MigrationRefusal::OwnAlias)
        );
        assert_eq!(MigrationRefusal::parse_misskey_code("unknown"), None);
    }

    #[test]
    fn misskey_spelling_of_an_alias_list_decodes_on_the_account() {
        let account: Account =
            serde_json::from_str(r#"{ "id": "1", "acct": "me", "alsoKnownAs": ["2", "3"] }"#)
                .expect("Misskey detailed user with aliases");
        assert_eq!(account.also_known_as, vec!["2".to_owned(), "3".to_owned()]);
    }
}
