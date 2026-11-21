//! The action traits for every transport, written once.
//!
//! Split from [`api`](super::api) because it answers a different question: that file
//! describes what the SDK *reads*, this one what a user *does*. Both are implemented
//! for the same `RestInstance<T>`, so a transport still has to write nothing.

use core::sync::atomic::{AtomicU64, Ordering};

use crate::actions::{AccountActions, Browse, MediaActions, Search, StatusActions};
use crate::model::{
    Account, Conversation, Draft, HistoryEntry, List, LocalId, MediaAttachment, Page, PageParams,
    Poll, ProfileField, ProfileUpdate, Relationship, SearchResults, Status,
};
use crate::rest::{RestInstance, Transport};
use crate::{Error, Result};

/// The JSON body a profile change sends when it carries metadata rows.
///
/// Nodus reads the rows as `fields`; Mastodon reads the same list as `fields_attributes`.
/// Sending both lets one request set rows on either server, and each ignores the name it
/// does not know. The other members are flattened from [`ProfileUpdate`], which omits the
/// ones that were left unset.
#[derive(serde::Serialize)]
struct ProfileCredentials<'a> {
    #[serde(flatten)]
    update: &'a ProfileUpdate,
    fields_attributes: &'a [ProfileField],
}

/// Runs a status verb that answers with the status itself.
async fn status_verb<T: Transport>(
    client: &RestInstance<T>,
    id: &str,
    verb: &str,
) -> Result<Status> {
    let url = client.url_segments(&["api", "v1", "statuses", id, verb])?;
    client.fetch(client.post(url)).await
}

impl<T: Transport> StatusActions for RestInstance<T> {
    type Error = Error;

    async fn favourite(&self, id: &str) -> Result<Status> {
        status_verb(self, id, "favourite").await
    }

    async fn unfavourite(&self, id: &str) -> Result<Status> {
        status_verb(self, id, "unfavourite").await
    }

    async fn reblog(&self, id: &str) -> Result<Status> {
        status_verb(self, id, "reblog").await
    }

    async fn unreblog(&self, id: &str) -> Result<Status> {
        status_verb(self, id, "unreblog").await
    }

    async fn bookmark(&self, id: &str) -> Result<Status> {
        status_verb(self, id, "bookmark").await
    }

    async fn unbookmark(&self, id: &str) -> Result<Status> {
        status_verb(self, id, "unbookmark").await
    }

    async fn pin(&self, id: &str) -> Result<Status> {
        status_verb(self, id, "pin").await
    }

    async fn unpin(&self, id: &str) -> Result<Status> {
        status_verb(self, id, "unpin").await
    }

    async fn edit_status(&self, id: &str, draft: &Draft) -> Result<Status> {
        let url = self.url_segments(&["api", "v1", "statuses", id])?;
        let form = super::status_form(draft, true);
        self.fetch(self.put_form(url, &form)).await
    }

    async fn vote(&self, poll_id: &str, choices: &[usize]) -> Result<Poll> {
        let url = self.url_segments(&["api", "v1", "polls", poll_id, "votes"])?;
        let form: Vec<(&str, String)> = choices
            .iter()
            .map(|choice| ("choices[]", choice.to_string()))
            .collect();
        self.fetch(self.post_form(url, &form)).await
    }

    async fn history(&self, id: &str) -> Result<Vec<HistoryEntry>> {
        let url = self.url_segments(&["api", "v1", "statuses", id, "history"])?;
        self.fetch(self.get(url)).await
    }

    async fn reblogged_by(
        &self,
        id: &str,
        params: PageParams,
    ) -> Result<Page<crate::model::Account>> {
        let url =
            self.page_url_segments(&["api", "v1", "statuses", id, "reblogged_by"], &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn favourited_by(
        &self,
        id: &str,
        params: PageParams,
    ) -> Result<Page<crate::model::Account>> {
        let url =
            self.page_url_segments(&["api", "v1", "statuses", id, "favourited_by"], &params)?;
        self.fetch_page(self.get(url), &params).await
    }
}

impl<T: Transport> AccountActions for RestInstance<T> {
    type Error = Error;

    async fn block(&self, id: &LocalId) -> Result<Relationship> {
        self.account_verb(&id.0, "block").await
    }

    async fn unblock(&self, id: &LocalId) -> Result<Relationship> {
        self.account_verb(&id.0, "unblock").await
    }

    async fn mute(&self, id: &LocalId) -> Result<Relationship> {
        self.account_verb(&id.0, "mute").await
    }

    async fn unmute(&self, id: &LocalId) -> Result<Relationship> {
        self.account_verb(&id.0, "unmute").await
    }

    async fn relationships(&self, ids: &[LocalId]) -> Result<Vec<Relationship>> {
        let mut url = self.url("/api/v1/accounts/relationships")?;
        {
            let mut query = url.query_pairs_mut();
            for id in ids {
                query.append_pair("id[]", &id.0);
            }
        }
        self.fetch(self.get(url)).await
    }

    async fn follow_requests(&self) -> Result<Vec<Account>> {
        let url = self.url("/api/v1/follow_requests")?;
        self.fetch(self.get(url)).await
    }

    async fn authorize_follow_request(&self, id: &LocalId) -> Result<Relationship> {
        let url = self.url_segments(&["api", "v1", "follow_requests", &id.0, "authorize"])?;
        self.fetch(self.post(url)).await
    }

    async fn reject_follow_request(&self, id: &LocalId) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "follow_requests", &id.0, "reject"])?;
        self.send_empty(self.post(url)).await
    }

    async fn mutes(&self, params: PageParams) -> Result<Page<Account>> {
        let url = self.page_url("/api/v1/mutes", &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn blocks(&self, params: PageParams) -> Result<Page<Account>> {
        let url = self.page_url("/api/v1/blocks", &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn update_profile(&self, update: &crate::model::ProfileUpdate) -> Result<Account> {
        let url = self.url("/api/v1/accounts/update_credentials")?;
        let upload = update.avatar_image.is_some() || update.header_image.is_some();
        // `fields` is a nested list, which the endpoint's form encoding cannot spell, so a
        // change that carries rows goes as JSON. An upload stays multipart because that
        // is the only encoding that can carry the image bytes.
        if let Some(fields) = update.fields.as_deref().filter(|_| !upload) {
            let body = ProfileCredentials {
                update,
                fields_attributes: fields,
            };
            let body =
                serde_json::to_vec(&body).map_err(|error| Error::Decode(error.to_string()))?;
            return self
                .fetch(self.patch_body(url, "application/json", body))
                .await;
        }
        let mut form: Vec<(&str, String)> = Vec::with_capacity(7);
        if let Some(display_name) = &update.display_name {
            form.push(("display_name", display_name.clone()));
        }
        if let Some(note) = &update.note {
            form.push(("note", note.clone()));
        }
        if let Some(locked) = update.locked {
            form.push(("locked", locked.to_string()));
        }
        if let Some(bot) = update.bot {
            form.push(("bot", bot.to_string()));
        }
        if let Some(discoverable) = update.discoverable {
            form.push(("discoverable", discoverable.to_string()));
        }
        if let Some(avatar) = &update.avatar {
            form.push(("avatar", avatar.clone()));
        }
        if let Some(header) = &update.header {
            form.push(("header", header.clone()));
        }
        let upload = update.avatar_image.is_some() || update.header_image.is_some();
        if upload {
            let mut parts: Vec<Part<'_>> = form
                .iter()
                .map(|(name, value)| Part::field(name, value))
                .collect();
            if let Some(image) = &update.avatar_image {
                parts.push(Part::file("avatar", "avatar", &image.mime, &image.bytes));
            }
            if let Some(image) = &update.header_image {
                parts.push(Part::file("header", "header", &image.mime, &image.bytes));
            }
            let (boundary, body) = multipart_body(&parts)?;
            let content_type = format!("multipart/form-data; boundary={boundary}");
            return self.fetch(self.patch_body(url, &content_type, body)).await;
        }
        self.fetch(self.patch_form(url, &form)).await
    }

    async fn followers(
        &self,
        id: &LocalId,
        params: PageParams,
    ) -> Result<Page<crate::model::Account>> {
        let url =
            self.page_url_segments(&["api", "v1", "accounts", &id.0, "followers"], &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn following(
        &self,
        id: &LocalId,
        params: PageParams,
    ) -> Result<Page<crate::model::Account>> {
        let url =
            self.page_url_segments(&["api", "v1", "accounts", &id.0, "following"], &params)?;
        self.fetch_page(self.get(url), &params).await
    }
}

/// Prefix for generated multipart boundaries; a unique suffix is appended and checked
/// against every part in `multipart_body`.
const UPLOAD_BOUNDARY_PREFIX: &str = "nodus-upload-boundary";
static UPLOAD_BOUNDARY_SEQ: AtomicU64 = AtomicU64::new(1);

impl<T: Transport> RestInstance<T> {
    /// The exact request `upload_media` would send.
    ///
    /// A browser can send this itself with `XMLHttpRequest` to watch the bytes leave
    /// the machine, which is the only way to show real upload progress — `fetch`, and
    /// therefore every HTTP client built on it, reports nothing until it is done.
    pub fn media_upload_request(
        &self,
        bytes: &[u8],
        mime: &str,
        description: Option<&str>,
    ) -> Result<crate::rest::HttpRequest> {
        let url = self.url("/api/v2/media")?;
        let mut parts = vec![Part::file("file", "upload", mime, bytes)];
        if let Some(text) = description {
            parts.push(Part::field("description", text));
        }
        let (boundary, body) = multipart_body(&parts)?;
        Ok(self.post_body(
            url,
            &format!("multipart/form-data; boundary={boundary}"),
            body,
        ))
    }
}

/// One part of a `multipart/form-data` body: a text field or an uploaded file.
enum Part<'a> {
    Field {
        name: &'a str,
        value: &'a str,
    },
    File {
        name: &'a str,
        filename: &'a str,
        mime: &'a str,
        bytes: &'a [u8],
    },
}

impl<'a> Part<'a> {
    fn field(name: &'a str, value: &'a str) -> Self {
        Self::Field { name, value }
    }

    fn file(name: &'a str, filename: &'a str, mime: &'a str, bytes: &'a [u8]) -> Self {
        Self::File {
            name,
            filename,
            mime,
            bytes,
        }
    }

    /// Whether `needle` occurs anywhere this part contributes to the body.
    ///
    /// A boundary that appears inside the payload would split the part in two, so every
    /// byte of every part has to be checked, not just the file contents.
    fn contains(&self, needle: &[u8]) -> bool {
        match self {
            Self::Field { name, value } => {
                contains_subslice(name.as_bytes(), needle)
                    || contains_subslice(value.as_bytes(), needle)
            }
            Self::File {
                name,
                filename,
                mime,
                bytes,
            } => {
                contains_subslice(bytes, needle)
                    || contains_subslice(mime.as_bytes(), needle)
                    || contains_subslice(filename.as_bytes(), needle)
                    || contains_subslice(name.as_bytes(), needle)
            }
        }
    }
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn choose_boundary(parts: &[Part<'_>]) -> String {
    loop {
        let seq = UPLOAD_BOUNDARY_SEQ.fetch_add(1, Ordering::Relaxed);
        let candidate = format!("{UPLOAD_BOUNDARY_PREFIX}-{seq}");
        if !parts.iter().any(|part| part.contains(candidate.as_bytes())) {
            return candidate;
        }
    }
}

/// Serialises `parts`, choosing a boundary none of them contains.
fn multipart_body(parts: &[Part<'_>]) -> Result<(String, Vec<u8>)> {
    for part in parts {
        if let Part::File { mime, .. } = part
            && (mime.trim().is_empty() || mime.bytes().any(|b| b.is_ascii_control()))
        {
            return Err(Error::Transport("invalid media MIME type".to_owned()));
        }
    }
    let boundary = choose_boundary(parts);
    let mut body = Vec::with_capacity(512);
    for part in parts {
        match part {
            Part::Field { name, value } => body.extend_from_slice(
                format!(
                    "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n"
                )
                .as_bytes(),
            ),
            Part::File {
                name,
                filename,
                mime,
                bytes,
            } => {
                body.extend_from_slice(
                    format!(
                        "--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: {mime}\r\n\r\n"
                    )
                    .as_bytes(),
                );
                body.extend_from_slice(bytes);
                body.extend_from_slice(b"\r\n");
            }
        }
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    Ok((boundary, body))
}

impl<T: Transport> RestInstance<T> {
    /// The six account verbs that all answer with a relationship.
    async fn account_verb(&self, id: &str, verb: &str) -> Result<Relationship> {
        let url = self.url_segments(&["api", "v1", "accounts", id, verb])?;
        self.fetch(self.post(url)).await
    }
}

impl<T: Transport> MediaActions for RestInstance<T> {
    type Error = Error;

    async fn upload_media(
        &self,
        bytes: &[u8],
        mime: &str,
        description: Option<&str>,
    ) -> Result<MediaAttachment> {
        let request = self.media_upload_request(bytes, mime, description)?;
        self.fetch(request).await
    }

    async fn update_media(&self, id: &str, description: &str) -> Result<MediaAttachment> {
        let url = self.url_segments(&["api", "v1", "media", id])?;
        let form = [("description", description.to_owned())];
        let request = self.put_form(url, &form);
        self.fetch(request).await
    }

    async fn set_media_sensitive(&self, id: &str, sensitive: bool) -> Result<MediaAttachment> {
        let url = self.url_segments(&["api", "v1", "media", id])?;
        let form = [("sensitive", sensitive.to_string())];
        let request = self.put_form(url, &form);
        self.fetch(request).await
    }

    async fn media_status(&self, id: &str) -> Result<MediaAttachment> {
        let url = self.url_segments(&["api", "v1", "media", id])?;
        self.fetch(self.get(url)).await
    }
}

impl<T: Transport> Browse for RestInstance<T> {
    type Error = Error;

    async fn favourites(&self, params: PageParams) -> Result<Page<Status>> {
        let url = self.page_url("/api/v1/favourites", &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn bookmarks(&self, params: PageParams) -> Result<Page<Status>> {
        let url = self.page_url("/api/v1/bookmarks", &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn conversations(&self, params: PageParams) -> Result<Page<Conversation>> {
        let url = self.page_url("/api/v1/conversations", &params)?;
        self.fetch_page(self.get(url), &params).await
    }

    async fn mark_conversation_read(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "conversations", id, "read"])?;
        self.send_empty(self.post(url)).await
    }

    async fn directory(&self, local_only: bool, offset: u32) -> Result<Vec<Account>> {
        // `order=active` asks for accounts that actually post; the default ordering is
        // by sign-up date, which surfaces empty profiles.
        let limit = <Self as Browse>::DIRECTORY_PAGE;
        let mut url = self.url("/api/v1/directory")?;
        url.query_pairs_mut()
            .append_pair("local", if local_only { "true" } else { "false" })
            .append_pair("order", "active")
            .append_pair("offset", &offset.to_string())
            .append_pair("limit", &limit.to_string());
        self.fetch(self.get(url)).await
    }

    async fn suggestions(&self, limit: u16) -> Result<Vec<crate::model::Suggestion>> {
        let mut url = self.url("/api/v2/suggestions")?;
        url.query_pairs_mut()
            .append_pair("limit", &limit.to_string());
        self.fetch(self.get(url)).await
    }

    async fn mute_conversation(&self, id: &str, muted: bool) -> Result<Conversation> {
        let url = self.url_segments(&["api", "v1", "conversations", id, "mute"])?;
        let request = if muted {
            self.post(url)
        } else {
            self.delete(url)
        };
        self.fetch(request).await
    }

    async fn lists(&self) -> Result<Vec<List>> {
        let url = self.url("/api/v1/lists")?;
        self.fetch(self.get(url)).await
    }

    async fn create_list(&self, title: &str) -> Result<List> {
        let url = self.url("/api/v1/lists")?;
        self.fetch(self.post_form(url, &[("title", title.to_owned())]))
            .await
    }

    async fn rename_list(&self, id: &str, title: &str) -> Result<List> {
        let url = self.url_segments(&["api", "v1", "lists", id])?;
        self.fetch(self.put_form(url, &[("title", title.to_owned())]))
            .await
    }

    async fn delete_list(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "lists", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn add_to_list(&self, list_id: &str, accounts: &[LocalId]) -> Result<()> {
        self.list_members(list_id, accounts, true).await
    }

    async fn remove_from_list(&self, list_id: &str, accounts: &[LocalId]) -> Result<()> {
        self.list_members(list_id, accounts, false).await
    }

    async fn list_accounts(&self, list_id: &str) -> Result<Vec<Account>> {
        let url = self.url_segments(&["api", "v1", "lists", list_id, "accounts"])?;
        self.fetch(self.get(url)).await
    }

    async fn list_timeline(&self, list_id: &str, params: PageParams) -> Result<Page<Status>> {
        <Self as crate::api::InstanceApi>::list_timeline(self, list_id, params).await
    }
}

impl<T: Transport> RestInstance<T> {
    /// Adds or removes accounts, which differ only in the verb.
    async fn list_members(&self, list_id: &str, accounts: &[LocalId], add: bool) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "lists", list_id, "accounts"])?;
        let form: Vec<(&str, String)> = accounts
            .iter()
            .map(|account| ("account_ids[]", account.0.clone()))
            .collect();
        let request = if add {
            self.post_form(url, &form)
        } else {
            self.delete_form(url, &form)
        };
        self.send_empty(request).await
    }
}

impl<T: Transport> Search for RestInstance<T> {
    type Error = Error;

    async fn search(&self, query: &str, resolve: bool) -> Result<SearchResults> {
        let mut url = self.url("/api/v2/search")?;
        {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("q", query);
            pairs.append_pair("resolve", if resolve { "true" } else { "false" });
        }
        self.fetch(self.get(url)).await
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::rest::{HttpRequest, HttpResponse};

    #[derive(Clone, Default)]
    struct Recording(Rc<RefCell<Vec<HttpRequest>>>);

    impl Transport for Recording {
        fn send(
            &self,
            request: HttpRequest,
        ) -> impl core::future::Future<Output = Result<HttpResponse>> {
            self.0.borrow_mut().push(request);
            core::future::ready(Ok(HttpResponse {
                status: 200,
                retry_after_s: None,
                body: b"[]".to_vec(),
            }))
        }
    }

    fn instance() -> (RestInstance<Recording>, Rc<RefCell<Vec<HttpRequest>>>) {
        let recording = Recording::default();
        let seen = Rc::clone(&recording.0);
        let instance = crate::Instance::parse("https://mastodon.example").expect("valid host");
        (
            RestInstance::with_transport(&instance, "token", recording),
            seen,
        )
    }

    fn block_on<F: core::future::Future>(future: F) -> F::Output {
        let mut future = core::pin::pin!(future);
        let mut context = core::task::Context::from_waker(core::task::Waker::noop());
        match future.as_mut().poll(&mut context) {
            core::task::Poll::Ready(output) => output,
            core::task::Poll::Pending => panic!("recording transport never pends"),
        }
    }

    #[test]
    fn multipart_rejects_crlf_and_empty_mime() {
        let (client, _) = instance();
        for bad_mime in [
            "",
            "   ",
            "image/png\r\nX-Injected: yes",
            "image/png\nX-Injected: yes",
            "image/png\r",
            "image/png\0",
        ] {
            assert!(
                client
                    .media_upload_request(b"pngbytes", bad_mime, None)
                    .is_err(),
                "expected bad mime {bad_mime:?} to be rejected"
            );
        }
    }

    #[test]
    fn multipart_boundary_never_collides_with_payload_or_description() {
        let (client, _) = instance();
        let payload = b"--nodus-upload-boundary\r\nContent-Disposition: form-data; name=\"evil\"\r\n\r\npwned";
        let desc = "--nodus-upload-boundary\r\n--nodus-upload-boundary--";
        let req = client
            .media_upload_request(payload, "image/png", Some(desc))
            .expect("upload request should choose a collision-free boundary");

        let content_type = req
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("Content-Type"))
            .map(|(_, value)| value.as_str())
            .expect("Content-Type header");
        let boundary = content_type
            .strip_prefix("multipart/form-data; boundary=")
            .expect("multipart boundary parameter");
        let body = req.body.expect("request body");

        assert!(
            !payload
                .windows(boundary.len())
                .any(|w| w == boundary.as_bytes()),
            "chosen boundary {boundary:?} must not appear in file payload"
        );
        assert!(
            !desc
                .as_bytes()
                .windows(boundary.len())
                .any(|w| w == boundary.as_bytes()),
            "chosen boundary {boundary:?} must not appear in description"
        );
        let delimiter = format!("--{boundary}");
        let delimiter_count = body
            .windows(delimiter.len())
            .filter(|w| *w == delimiter.as_bytes())
            .count();
        assert_eq!(
            delimiter_count, 3,
            "expected exactly 3 boundary markers (file, description, closing), got {delimiter_count}"
        );
    }

    #[test]
    fn action_urls_escape_ids_and_build_directory_query_safely() {
        let (client, seen) = instance();
        let _ = block_on(client.favourite("../admin?x=1#y"));
        let _ = block_on(client.list_timeline("../x#y", PageParams::default()));
        let _ = block_on(client.directory(true, 40));

        let reqs = seen.borrow();
        assert_eq!(
            reqs[0].url.path(),
            "/api/v1/statuses/..%2Fadmin%3Fx=1%23y/favourite"
        );
        assert_eq!(reqs[0].url.query(), None);
        assert_eq!(reqs[0].url.fragment(), None);
        assert_eq!(reqs[1].url.path(), "/api/v1/timelines/list/..%2Fx%23y");
        assert_eq!(reqs[2].url.path(), "/api/v1/directory");
        assert_eq!(
            reqs[2].url.query(),
            Some("local=true&order=active&offset=40&limit=40")
        );
    }
}
