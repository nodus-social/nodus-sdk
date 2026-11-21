//! Pages that are not timelines: notifications, messages, saved posts, people, search.
//!
//! A timeline is a list of ids into the entity store, because a status must look the same
//! in every screen. Everything else here is a page of finished objects: a notification, a
//! conversation or a search result has no second copy to stay in step with.

use std::collections::HashSet;

use nodus_client_core::model::{
    Account, Conversation, FeaturedTag, Filter, Notification, ScheduledStatus, SearchResults,
    Status,
};
use serde::{Deserialize, Serialize};

/// Which page a stored entry belongs to.
///
/// The account and the query are part of the key: "the followers of A" and "the followers
/// of B" are different pages, and a search for one word says nothing about another.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PageKind {
    Notifications,
    Conversations,
    Favourites,
    Bookmarks,
    Followers(String),
    Following(String),
    Mutes,
    Blocks,
    Scheduled,
    Search(String),
    /// The viewer's keyword filters.
    Filters,
    /// Hashtags pinned to the viewer's profile.
    FeaturedTags,
    /// The suggestion list, keyed by whether it is limited to this instance.
    ///
    /// The instance pages this list by offset rather than by id, so its cursor is a
    /// number written as text — see `Sdk::ensure_page`.
    Directory(bool),
}

/// The contents of one page.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum PageItems {
    #[default]
    Empty,
    Statuses(Vec<Status>),
    Notifications(Vec<Notification>),
    Accounts(Vec<Account>),
    Conversations(Vec<Conversation>),
    Scheduled(Vec<ScheduledStatus>),
    Filters(Vec<Filter>),
    FeaturedTags(Vec<FeaturedTag>),
    Search(Box<SearchResults>),
}

impl PageItems {
    /// The ids this page holds, in order. Search results have none: they are replaced whole.
    fn ids(&self) -> Vec<String> {
        match self {
            Self::Empty | Self::Search(_) => Vec::new(),
            Self::Statuses(items) => items.iter().map(|item| item.id.clone()).collect(),
            Self::Notifications(items) => items.iter().map(|item| item.id.clone()).collect(),
            Self::Accounts(items) => items.iter().map(|item| item.id.clone()).collect(),
            Self::Conversations(items) => items.iter().map(|item| item.id.clone()).collect(),
            Self::Scheduled(items) => items.iter().map(|item| item.id.clone()).collect(),
            Self::Filters(items) => items.iter().map(|item| item.id.clone()).collect(),
            Self::FeaturedTags(items) => items.iter().map(|item| item.id.clone()).collect(),
        }
    }

    /// Whether this page speaks the same language as `other`.
    #[must_use]
    pub fn same_shape(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

/// One stored page: its items, the cursor for older ones, and when it was fetched.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PageEntry {
    pub items: PageItems,
    pub cursor: Option<String>,
    pub fetched_at: Option<i64>,
}

impl PageEntry {
    /// Folds an older page into this one, keeping display order.
    ///
    /// Ids already held are dropped: an instance that shifts between requests would
    /// otherwise show the same notification twice, and a keyed list treats a duplicate
    /// key as a panic rather than a blemish.
    pub fn append(&mut self, older: Self) {
        if !self.items.same_shape(&older.items) {
            return;
        }
        let known: HashSet<String> = self.items.ids().into_iter().collect();
        let older_ids = older.items.ids();
        let keep: Vec<bool> = older_ids.iter().map(|id| !known.contains(id)).collect();

        match (&mut self.items, older.items) {
            (PageItems::Statuses(mine), PageItems::Statuses(theirs)) => {
                extend_unique(mine, theirs, &keep);
            }
            (PageItems::Notifications(mine), PageItems::Notifications(theirs)) => {
                extend_unique(mine, theirs, &keep);
            }
            (PageItems::Accounts(mine), PageItems::Accounts(theirs)) => {
                extend_unique(mine, theirs, &keep);
            }
            (PageItems::Conversations(mine), PageItems::Conversations(theirs)) => {
                extend_unique(mine, theirs, &keep);
            }
            (PageItems::Scheduled(mine), PageItems::Scheduled(theirs)) => {
                extend_unique(mine, theirs, &keep);
            }
            (PageItems::Filters(mine), PageItems::Filters(theirs)) => {
                extend_unique(mine, theirs, &keep);
            }
            (PageItems::FeaturedTags(mine), PageItems::FeaturedTags(theirs)) => {
                extend_unique(mine, theirs, &keep);
            }
            _ => {}
        }
        self.cursor = older.cursor;
    }

    /// Drops references to a deleted status from this page.
    pub fn remove_status(&mut self, id: &str) {
        let matches = |status: &Status| {
            status.id == id || status.reblog.as_ref().is_some_and(|inner| inner.id == id)
        };
        match &mut self.items {
            PageItems::Statuses(items) => items.retain(|status| !matches(status)),
            PageItems::Notifications(items) => {
                items.retain(|note| note.status.as_ref().is_none_or(|status| !matches(status)));
            }
            PageItems::Conversations(items) => {
                for conversation in items {
                    if conversation.last_status.as_ref().is_some_and(matches) {
                        conversation.last_status = None;
                    }
                }
            }
            PageItems::Search(results) => results.statuses.retain(|status| !matches(status)),
            PageItems::Empty
            | PageItems::Accounts(_)
            | PageItems::Scheduled(_)
            | PageItems::Filters(_)
            | PageItems::FeaturedTags(_) => {}
        }
    }

    /// Whether this page is younger than `ttl_ms`, measured at `now`.
    #[must_use]
    pub fn is_fresh(&self, now: i64, ttl_ms: i64) -> bool {
        self.fetched_at
            .is_some_and(|fetched| now.saturating_sub(fetched) < ttl_ms)
    }
}

fn extend_unique<T>(mine: &mut Vec<T>, theirs: Vec<T>, keep: &[bool]) {
    mine.extend(
        theirs
            .into_iter()
            .enumerate()
            .filter_map(|(index, item)| keep.get(index).copied().unwrap_or(false).then_some(item)),
    );
}

#[cfg(test)]
mod tests {
    use nodus_client_core::model::{FilterAction, NotificationKind, Visibility};

    use super::*;

    fn account(id: &str) -> Account {
        Account {
            id: id.to_owned(),
            acct: "someone@remote.test".to_owned(),
            display_name: "Someone".to_owned(),
            note: String::new(),
            avatar: String::new(),
            header: String::new(),
            bot: false,
            locked: false,
            followers_count: 0,
            following_count: 0,
            statuses_count: 0,
            last_status_at: None,
            discoverable: None,
            indexable: None,
            noindex: None,
            fields: Vec::new(),
        }
    }

    fn status(id: &str) -> Status {
        Status {
            id: id.to_owned(),
            created_at: "2026-11-07T00:00:00.000Z".to_owned(),
            content: "<p>hello</p>".to_owned(),
            visibility: Visibility::Public,
            account: account(&format!("acct-{id}")),
            favourites_count: 0,
            reblogs_count: 0,
            replies_count: 0,
            favourited: false,
            reblogged: false,
            bookmarked: false,
            reblog: None,
            in_reply_to_id: None,
            emojis: Vec::new(),
            url: None,
            sensitive: false,
            spoiler_text: String::new(),
            language: None,
            content_format: None,
            content_source: None,
            media_attachments: Vec::new(),
            mentions: Vec::new(),
            tags: Vec::new(),
            poll: None,
            edited_at: None,
            pinned: None,
        }
    }

    fn notification(id: &str, related: Option<Status>) -> Notification {
        Notification {
            id: id.to_owned(),
            kind: NotificationKind::Mention,
            created_at: "2026-11-07T00:00:00.000Z".to_owned(),
            account: status("author").account,
            status: related,
        }
    }

    #[test]
    fn append_clears_cursor_when_final_page_has_none() {
        let mut page = PageEntry {
            items: PageItems::Statuses(vec![status("1")]),
            cursor: Some("c1".to_owned()),
            fetched_at: Some(100),
        };
        page.append(PageEntry {
            items: PageItems::Statuses(vec![status("2")]),
            cursor: None,
            fetched_at: Some(200),
        });

        assert_eq!(page.cursor, None);
        assert_eq!(page.items.ids(), vec!["1", "2"]);
    }

    #[test]
    fn append_handles_filters_and_featured_tags_with_dedup() {
        let filter = |id: &str| Filter {
            id: id.to_owned(),
            title: format!("filter-{id}"),
            context: Vec::new(),
            expires_at: None,
            filter_action: FilterAction::Warn,
            keywords: Vec::new(),
        };
        let mut filters = PageEntry {
            items: PageItems::Filters(vec![filter("f1")]),
            cursor: Some("c1".to_owned()),
            fetched_at: Some(1),
        };
        filters.append(PageEntry {
            items: PageItems::Filters(vec![filter("f1"), filter("f2")]),
            cursor: None,
            fetched_at: Some(2),
        });
        assert_eq!(filters.items.ids(), vec!["f1", "f2"]);
        assert_eq!(filters.cursor, None);

        let tag = |id: &str| FeaturedTag {
            id: id.to_owned(),
            name: format!("tag-{id}"),
            statuses_count: Some(1),
            last_status_at: None,
        };
        let mut tags = PageEntry {
            items: PageItems::FeaturedTags(vec![tag("t1")]),
            cursor: Some("c1".to_owned()),
            fetched_at: Some(1),
        };
        tags.append(PageEntry {
            items: PageItems::FeaturedTags(vec![tag("t1"), tag("t2")]),
            cursor: Some("c2".to_owned()),
            fetched_at: Some(2),
        });
        assert_eq!(tags.items.ids(), vec!["t1", "t2"]);
        assert_eq!(tags.cursor.as_deref(), Some("c2"));
    }

    #[test]
    fn remove_status_prunes_all_status_bearing_pages() {
        let mut statuses = PageEntry {
            items: PageItems::Statuses(vec![status("keep"), status("del")]),
            cursor: None,
            fetched_at: None,
        };
        statuses.remove_status("del");
        assert_eq!(statuses.items.ids(), vec!["keep"]);

        let mut notifications = PageEntry {
            items: PageItems::Notifications(vec![
                notification("n-follow", None),
                notification("n-del", Some(status("del"))),
                notification("n-keep", Some(status("keep"))),
            ]),
            cursor: None,
            fetched_at: None,
        };
        notifications.remove_status("del");
        assert_eq!(notifications.items.ids(), vec!["n-follow", "n-keep"]);

        let mut conversations = PageEntry {
            items: PageItems::Conversations(vec![Conversation {
                id: "conv-1".to_owned(),
                unread: true,
                accounts: Vec::new(),
                last_status: Some(status("del")),
            }]),
            cursor: None,
            fetched_at: None,
        };
        conversations.remove_status("del");
        if let PageItems::Conversations(items) = &conversations.items {
            assert!(items[0].last_status.is_none());
        } else {
            panic!("expected conversations");
        }

        let mut search = PageEntry {
            items: PageItems::Search(Box::new(SearchResults {
                accounts: Vec::new(),
                statuses: vec![status("del"), status("keep")],
                hashtags: Vec::new(),
            })),
            cursor: None,
            fetched_at: None,
        };
        search.remove_status("del");
        if let PageItems::Search(results) = &search.items {
            assert_eq!(
                results
                    .statuses
                    .iter()
                    .map(|s| s.id.as_str())
                    .collect::<Vec<_>>(),
                vec!["keep"]
            );
        } else {
            panic!("expected search");
        }
    }
}
