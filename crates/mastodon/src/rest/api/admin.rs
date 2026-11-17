//! Moderation over the REST transport.
//!
//! Mastodon splits moderation across two prefixes, and the split is not by version:
//! reports, accounts, blocks, allows and scanners live under `/api/v1/admin`, while the
//! admin console's meta and overview live beside them. A wrong prefix does not answer 404
//! as JSON — it falls through to the instance's HTML error page, which reads like "this
//! account cannot moderate" and cost a debugging session, so every path below is spelled
//! out rather than derived.
//!
//! Where an endpoint accepts either form or JSON, the encoding follows what the server's
//! extractor actually reads: the moderation verbs take forms, the structured writes
//! (staff flags, notes, scanner definitions) take JSON, and the asset upload is JSON
//! because its payload is base64.

use crate::actions::AdminActions;
use crate::model::{
    AccountDetail, AccountNote, AdminAccount, AdminAccountQuery, AdminAlert, AdminMeasure,
    AdminMeta, AdminOverview, Announcement, AnnouncementDraft, AssetUpload, AuditEntry,
    DomainAllow, DomainBlock, DomainBlockDraft, DomainImportOutcome, DomainSeverity,
    EmailDomainBlock, FlagState, ModerationAction, NewScanner, NodeLog, Report, ReportDraft,
    ReportFilter, ReportState, Scan, ScanFlag, ScanRequest, ScanSeverity, Scanner, ScannerAsset,
    ScannerRule, Topology, WarningPreset,
};
use crate::rest::{RestInstance, Transport};
use crate::{Error, Result};
use url::Url;

/// Adds every pair to a URL's query, in the order given.
fn with_query(mut url: Url, pairs: &[(&str, String)]) -> Url {
    url.query_pairs_mut().extend_pairs(pairs);
    url
}

/// Adds a pair only when the caller actually set something.
///
/// An absent filter must mean "do not narrow". Sending an empty `state=` would ask the
/// server for reports whose state is the empty string, which is none of them.
fn push(pairs: &mut Vec<(&str, String)>, key: &'static str, value: Option<&String>) {
    if let Some(value) = value {
        pairs.push((key, value.clone()));
    }
}

/// Serialises a request body, keeping the failure inside this crate's error type.
fn json_body<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|error| Error::Decode(error.to_string()))
}

/// A count the scan endpoint answers with.
#[derive(serde::Deserialize)]
struct Queued {
    #[serde(default)]
    queued: u64,
}

impl<T: Transport> AdminActions for RestInstance<T> {
    type Error = Error;

    async fn reports(&self, state: ReportState, limit: u16) -> Result<Vec<Report>> {
        let url = self.url("/api/v1/admin/reports")?;
        let pairs = [
            ("state", state.as_str().to_owned()),
            ("limit", limit.to_string()),
        ];
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn reports_page(&self, filter: &ReportFilter) -> Result<Vec<Report>> {
        let url = self.url("/api/v1/admin/reports")?;
        let mut pairs = Vec::new();
        push(
            &mut pairs,
            "state",
            filter.state.map(|state| state.as_str().to_owned()).as_ref(),
        );
        push(&mut pairs, "category", filter.category.as_ref());
        push(
            &mut pairs,
            "target_account_id",
            filter.target_account_id.as_ref(),
        );
        push(&mut pairs, "account_id", filter.account_id.as_ref());
        push(
            &mut pairs,
            "assigned_account_id",
            filter.assigned_account_id.as_ref(),
        );
        if filter.unassigned {
            pairs.push(("unassigned", "true".to_owned()));
        }
        push(
            &mut pairs,
            "limit",
            filter.limit.map(|limit| limit.to_string()).as_ref(),
        );
        push(
            &mut pairs,
            "offset",
            filter.offset.map(|offset| offset.to_string()).as_ref(),
        );
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn report(&self, id: &str) -> Result<Report> {
        let url = self.url_segments(&["api", "v1", "admin", "reports", id])?;
        self.fetch(self.get(url)).await
    }

    async fn resolve_report(&self, id: &str) -> Result<Report> {
        let url = self.url_segments(&["api", "v1", "admin", "reports", id, "resolve"])?;
        self.fetch(self.post(url)).await
    }

    async fn resolve_report_with_note(&self, id: &str, note: &str) -> Result<Report> {
        let url = self.url_segments(&["api", "v1", "admin", "reports", id, "resolve"])?;
        let body = serde_json::json!({ "note": note });
        let request = self.post_body(url, "application/json", body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn reopen_report(&self, id: &str) -> Result<Report> {
        let url = self.url_segments(&["api", "v1", "admin", "reports", id, "reopen"])?;
        self.fetch(self.post(url)).await
    }

    async fn assign_report(&self, id: &str, account_id: Option<&str>) -> Result<Report> {
        let url = self.url_segments(&["api", "v1", "admin", "reports", id, "assign"])?;
        // `null` is how the queue is told to unclaim; omitting the key would leave the
        // current moderator in place.
        let body = serde_json::json!({ "account_id": account_id });
        let request = self.post_body(url, "application/json", body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn create_report(&self, draft: &ReportDraft) -> Result<Report> {
        let url = self.url("/api/v1/reports")?;
        // The server reads this one as a form, and lists travel comma-separated.
        let mut form = vec![("account_id", draft.account_id.clone())];
        if !draft.status_ids.is_empty() {
            form.push(("status_ids", draft.status_ids.join(",")));
        }
        if !draft.category.is_empty() {
            form.push(("category", draft.category.clone()));
        }
        if !draft.comment.is_empty() {
            form.push(("comment", draft.comment.clone()));
        }
        if !draft.rule_ids.is_empty() {
            form.push(("rule_ids", draft.rule_ids.join(",")));
        }
        self.fetch(self.post_form(url, &form)).await
    }

    async fn admin_accounts(&self, local: bool, limit: u16) -> Result<Vec<AdminAccount>> {
        let url = self.url("/api/v1/admin/accounts")?;
        let pairs = [
            ("origin", if local { "local" } else { "remote" }.to_owned()),
            ("limit", limit.to_string()),
        ];
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn search_accounts(&self, query: &AdminAccountQuery) -> Result<Vec<AdminAccount>> {
        let url = self.url("/api/v1/admin/accounts")?;
        let mut pairs = Vec::new();
        push(
            &mut pairs,
            "origin",
            query
                .origin
                .map(|origin| origin.as_str().to_owned())
                .as_ref(),
        );
        push(
            &mut pairs,
            "status",
            query
                .status
                .map(|status| status.as_str().to_owned())
                .as_ref(),
        );
        push(&mut pairs, "username", query.username.as_ref());
        push(&mut pairs, "display_name", query.display_name.as_ref());
        push(&mut pairs, "email", query.email.as_ref());
        push(&mut pairs, "by_domain", query.by_domain.as_ref());
        if query.staff {
            pairs.push(("staff", "true".to_owned()));
        }
        push(
            &mut pairs,
            "limit",
            query.limit.map(|limit| limit.to_string()).as_ref(),
        );
        push(
            &mut pairs,
            "offset",
            query.offset.map(|offset| offset.to_string()).as_ref(),
        );
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn moderate_account(&self, id: &str, action: ModerationAction) -> Result<AdminAccount> {
        let url = self.url_segments(&["api", "v1", "admin", "accounts", id, "action"])?;
        let form = vec![("type", action.as_str().to_owned())];
        self.fetch(self.post_form(url, &form)).await
    }

    async fn moderate_account_with_note(
        &self,
        id: &str,
        action: ModerationAction,
        note: &str,
        report_id: Option<&str>,
    ) -> Result<AdminAccount> {
        let url = self.url_segments(&["api", "v1", "admin", "accounts", id, "action"])?;
        let mut form = vec![
            ("type", action.as_str().to_owned()),
            ("text", note.to_owned()),
        ];
        if let Some(report_id) = report_id {
            form.push(("report_id", report_id.to_owned()));
        }
        self.fetch(self.post_form(url, &form)).await
    }

    async fn set_staff(&self, id: &str, staff: bool) -> Result<AdminAccount> {
        let url = self.url_segments(&["api", "v1", "admin", "accounts", id, "staff"])?;
        let body = serde_json::json!({ "staff": staff });
        let request = self.post_body(url, "application/json", body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn account_detail(&self, id: &str) -> Result<AccountDetail> {
        let url = self.url_segments(&["api", "v1", "admin", "accounts", id])?;
        self.fetch(self.get(url)).await
    }

    async fn add_account_note(&self, id: &str, body: &str) -> Result<AccountNote> {
        let url = self.url_segments(&["api", "v1", "admin", "accounts", id, "notes"])?;
        // A note is user-visible prose, so it travels as JSON rather than in a form where
        // an unescaped `&` would truncate it.
        let body = serde_json::json!({ "body": body });
        let request = self.post_body(url, "application/json", body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn delete_account_note(&self, id: &str, note_id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "accounts", id, "notes", note_id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn purge_account(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "accounts", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn set_lossless(&self, id: &str, images: bool, videos: bool) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "accounts", id, "lossless"])?;
        let body = serde_json::json!({ "images": images, "videos": videos });
        let request = self.post_body(url, "application/json", body.to_string().into_bytes());
        self.send_empty(request).await
    }

    async fn domain_blocks(&self, limit: u16) -> Result<Vec<DomainBlock>> {
        let url = self.url("/api/v1/admin/domain_blocks")?;
        let pairs = [("limit", limit.to_string())];
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn create_domain_block(
        &self,
        domain: &str,
        severity: DomainSeverity,
    ) -> Result<DomainBlock> {
        let url = self.url("/api/v1/admin/domain_blocks")?;
        let form = vec![
            ("domain", domain.to_owned()),
            ("severity", severity.as_str().to_owned()),
        ];
        self.fetch(self.post_form(url, &form)).await
    }

    async fn update_domain_block(&self, id: &str, draft: &DomainBlockDraft) -> Result<DomainBlock> {
        let url = self.url_segments(&["api", "v1", "admin", "domain_blocks", id])?;
        let form = vec![
            ("domain", draft.domain.clone()),
            ("severity", draft.severity.as_str().to_owned()),
            ("reject_media", draft.reject_media.to_string()),
            ("reject_reports", draft.reject_reports.to_string()),
            ("obfuscate", draft.obfuscate.to_string()),
            ("public_comment", draft.public_comment.clone()),
        ];
        self.fetch(self.put_form(url, &form)).await
    }

    async fn delete_domain_block(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "domain_blocks", id])?;
        // The body of a delete is empty, so it cannot be decoded as the removed block.
        self.send_empty(self.delete(url)).await
    }

    async fn import_domain_blocks(
        &self,
        domains: &[&str],
        severity: DomainSeverity,
    ) -> Result<DomainImportOutcome> {
        let url = self.url_segments(&["api", "v1", "admin", "domain_blocks", "import"])?;
        let body = serde_json::json!({
            "domains": domains,
            "severity": severity.as_str(),
        });
        let request = self.post_body(url, "application/json", body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn domain_allows(&self, limit: u16) -> Result<Vec<DomainAllow>> {
        let url = self.url("/api/v1/admin/domain_allows")?;
        let pairs = [("limit", limit.to_string())];
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn create_domain_allow(&self, domain: &str) -> Result<DomainAllow> {
        let url = self.url("/api/v1/admin/domain_allows")?;
        let form = vec![("domain", domain.to_owned())];
        self.fetch(self.post_form(url, &form)).await
    }

    async fn delete_domain_allow(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "domain_allows", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn email_domain_blocks(&self, limit: u16) -> Result<Vec<EmailDomainBlock>> {
        let url = self.url("/api/v1/admin/email_domain_blocks")?;
        let pairs = [("limit", limit.to_string())];
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn create_email_domain_block(
        &self,
        domain: &str,
        allow_registration: bool,
    ) -> Result<EmailDomainBlock> {
        let url = self.url("/api/v1/admin/email_domain_blocks")?;
        let form = vec![
            ("domain", domain.to_owned()),
            ("allow_registration", allow_registration.to_string()),
        ];
        self.fetch(self.post_form(url, &form)).await
    }

    async fn delete_email_domain_block(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "email_domain_blocks", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn measures(
        &self,
        keys: &[&str],
        start_at: &str,
        end_at: &str,
    ) -> Result<Vec<AdminMeasure>> {
        // A GET with a comma-separated `keys`, not a JSON body. The keys are a list on
        // the query string because that is the only verb this route answers: the same
        // path refuses a POST, and a POST to the v2 spelling answers the instance's HTML
        // 404 page, which looks like a scope problem.
        let url = self.url("/api/v1/admin/measures")?;
        let pairs = [
            ("keys", keys.join(",")),
            ("start_at", start_at.to_owned()),
            ("end_at", end_at.to_owned()),
        ];
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn audit(&self, limit: u16) -> Result<Vec<AuditEntry>> {
        let url = self.url("/api/v1/admin/audit")?;
        let pairs = [("limit", limit.to_string())];
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn admin_meta(&self) -> Result<AdminMeta> {
        let url = self.url("/api/v1/admin/meta")?;
        self.fetch(self.get(url)).await
    }

    async fn admin_overview(&self) -> Result<AdminOverview> {
        let url = self.url("/api/v1/admin/overview")?;
        self.fetch(self.get(url)).await
    }

    async fn admin_alerts(&self, unacknowledged: bool, limit: u16) -> Result<Vec<AdminAlert>> {
        let url = self.url("/api/v1/admin/alerts")?;
        let pairs = [
            ("unacknowledged", unacknowledged.to_string()),
            ("limit", limit.to_string()),
        ];
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn acknowledge_alert(&self, id: &str) -> Result<AdminAlert> {
        let url = self.url_segments(&["api", "v1", "admin", "alerts", id, "ack"])?;
        self.fetch(self.post(url)).await
    }

    async fn scanners(&self) -> Result<Vec<Scanner>> {
        let url = self.url("/api/v1/admin/scanners")?;
        self.fetch(self.get(url)).await
    }

    async fn create_scanner(&self, scanner: &NewScanner) -> Result<Scanner> {
        let url = self.url("/api/v1/admin/scanners")?;
        // Definitions are structured, so they travel as JSON; the secret is `null` when
        // absent, which the server treats as "no secret", not as "keep the old one".
        let request = self.post_body(url, "application/json", json_body(scanner)?);
        self.fetch(request).await
    }

    async fn update_scanner(&self, id: &str, scanner: &NewScanner) -> Result<Scanner> {
        let url = self.url_segments(&["api", "v1", "admin", "scanners", id])?;
        // Both spellings update on the server; PUT says which one this is.
        let request = self.put_body(url, json_body(scanner)?);
        self.fetch(request).await
    }

    async fn delete_scanner(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "scanners", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn refresh_scanner(&self, id: &str) -> Result<Scanner> {
        let url = self.url_segments(&["api", "v1", "admin", "scanners", id, "refresh"])?;
        self.fetch(self.post(url)).await
    }

    async fn set_scanner_rule(
        &self,
        scanner_id: &str,
        label: &str,
        threshold: f64,
        severity: ScanSeverity,
    ) -> Result<ScannerRule> {
        let url = self.url_segments(&["api", "v1", "admin", "scanners", scanner_id, "rules"])?;
        let body = serde_json::json!({
            "label": label,
            "threshold": threshold,
            "severity": severity.as_str(),
        });
        let request = self.put_body(url, body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn delete_scanner_rule(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "scanner_rules", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn scanner_assets(&self) -> Result<Vec<ScannerAsset>> {
        let url = self.url("/api/v1/admin/scanner_assets")?;
        self.fetch(self.get(url)).await
    }

    async fn upload_scanner_asset(&self, asset: &AssetUpload) -> Result<ScannerAsset> {
        let url = self.url("/api/v1/admin/scanner_assets")?;
        // Base64 in JSON, because the upload comes from a browser that has the bytes in
        // memory and no filesystem the server can see.
        let request = self.post_body(url, "application/json", json_body(asset)?);
        self.fetch(request).await
    }

    async fn delete_scanner_asset(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "scanner_assets", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn download_scanner_asset(&self, id: &str) -> Result<Vec<u8>> {
        let url = self.url_segments(&["api", "v1", "admin", "scanner_assets", id, "download"])?;
        // The one endpoint that answers bytes rather than JSON.
        self.fetch_bytes(self.get(url)).await
    }

    async fn scan_flags(&self, state: Option<FlagState>, limit: u16) -> Result<Vec<ScanFlag>> {
        let url = self.url("/api/v1/admin/flags")?;
        let mut pairs = Vec::new();
        push(
            &mut pairs,
            "state",
            state.map(|state| state.as_str().to_owned()).as_ref(),
        );
        pairs.push(("limit", limit.to_string()));
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn resolve_scan_flag(&self, id: &str, state: FlagState) -> Result<ScanFlag> {
        let url = self.url_segments(&["api", "v1", "admin", "flags", id, "resolve"])?;
        let body = serde_json::json!({ "state": state.as_str() });
        let request = self.post_body(url, "application/json", body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn scan_queue(&self, state: Option<&str>, limit: u16) -> Result<Vec<Scan>> {
        let url = self.url("/api/v1/admin/scans")?;
        let mut pairs = Vec::new();
        if let Some(state) = state {
            pairs.push(("state", state.to_owned()));
        }
        pairs.push(("limit", limit.to_string()));
        self.fetch(self.get(with_query(url, &pairs))).await
    }

    async fn enqueue_scan(&self, request: &ScanRequest) -> Result<u64> {
        let url = self.url("/api/v1/admin/scans")?;
        let body = json_body(request)?;
        let queued: Queued = self
            .fetch(self.post_body(url, "application/json", body))
            .await?;
        Ok(queued.queued)
    }

    async fn warning_presets(&self) -> Result<Vec<WarningPreset>> {
        let url = self.url("/api/v1/admin/warning_presets")?;
        self.fetch(self.get(url)).await
    }

    async fn create_warning_preset(&self, title: &str, text: &str) -> Result<WarningPreset> {
        let url = self.url("/api/v1/admin/warning_presets")?;
        let body = serde_json::json!({ "title": title, "text": text });
        let request = self.post_body(url, "application/json", body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn update_warning_preset(
        &self,
        id: &str,
        title: &str,
        text: &str,
    ) -> Result<WarningPreset> {
        let url = self.url_segments(&["api", "v1", "admin", "warning_presets", id])?;
        let body = serde_json::json!({ "title": title, "text": text });
        let request = self.put_body(url, body.to_string().into_bytes());
        self.fetch(request).await
    }

    async fn delete_warning_preset(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "warning_presets", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn admin_announcements(&self) -> Result<Vec<Announcement>> {
        let url = self.url("/api/v1/admin/announcements")?;
        self.fetch(self.get(url)).await
    }

    async fn create_announcement(&self, draft: AnnouncementDraft) -> Result<Announcement> {
        let url = self.url("/api/v1/admin/announcements")?;
        let form = announcement_form(&draft);
        self.fetch(self.post_form(url, &form)).await
    }

    async fn update_announcement(
        &self,
        id: &str,
        draft: AnnouncementDraft,
    ) -> Result<Announcement> {
        let url = self.url_segments(&["api", "v1", "admin", "announcements", id])?;
        let form = announcement_form(&draft);
        self.fetch(self.put_form(url, &form)).await
    }

    async fn delete_announcement(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "admin", "announcements", id])?;
        self.send_empty(self.delete(url)).await
    }

    async fn topology(&self) -> Result<Topology> {
        let url = self.url("/api/v1/admin/topology")?;
        self.fetch(self.get(url)).await
    }

    async fn node_logs(&self, name: &str, min_level: &str, limit: u16) -> Result<Vec<NodeLog>> {
        let url = self.url_segments(&["api", "v1", "admin", "topology", name, "logs"])?;
        let pairs = [
            ("min_level", min_level.to_owned()),
            ("limit", limit.to_string()),
        ];
        self.fetch(self.get(with_query(url, &pairs))).await
    }
}

/// Encodes an announcement as the server's form extractor reads it.
///
/// The dates are sent only when set: an omitted field is the server's way of clearing
/// one, and an empty string would fail its timestamp parse.
fn announcement_form(draft: &AnnouncementDraft) -> Vec<(&'static str, String)> {
    let mut form = vec![
        ("content", draft.content.clone()),
        ("published", draft.published.to_string()),
        ("all_day", draft.all_day.to_string()),
    ];
    if let Some(starts_at) = &draft.starts_at {
        form.push(("starts_at", starts_at.clone()));
    }
    if let Some(ends_at) = &draft.ends_at {
        form.push(("ends_at", ends_at.clone()));
    }
    form
}


#[cfg(test)]
mod tests {
    use core::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::rest::{HttpRequest, HttpResponse};

    /// One recorded request: verb, absolute URL, decoded body.
    type Sent = (String, String, Option<String>);

    /// Records `(verb, url, body)` of every request and answers with an empty list.
    ///
    /// The point of this fake is the *path*, not the payload: a wrong admin prefix
    /// answers with the instance's HTML 404 page, which looks exactly like a permission
    /// problem to whoever is holding the token.
    #[derive(Clone, Default)]
    struct Recording(Rc<RefCell<Vec<Sent>>>);

    impl Transport for Recording {
        fn send(
            &self,
            request: HttpRequest,
        ) -> impl core::future::Future<Output = Result<HttpResponse>> {
            self.0.borrow_mut().push((
                request.method.as_str().to_owned(),
                request.url.to_string(),
                request
                    .body
                    .as_ref()
                    .map(|bytes| String::from_utf8_lossy(bytes).into_owned()),
            ));
            core::future::ready(Ok(HttpResponse {
                status: 200,
                retry_after_s: None,
                body: b"[]".to_vec(),
            }))
        }
    }

    fn instance() -> (RestInstance<Recording>, Rc<RefCell<Vec<Sent>>>) {
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
            core::task::Poll::Pending => panic!("the recording transport never pends"),
        }
    }

    #[test]
    fn reports_are_asked_where_the_server_serves_them() {
        let (instance, seen) = instance();
        block_on(instance.reports(ReportState::Open, 1)).expect("recorded");
        let first = seen.borrow()[0].1.clone();
        assert!(
            first.starts_with("https://mastodon.example/api/v1/admin/reports?state=open"),
            "reports are `/api/v1/admin/reports`, got {first}"
        );
    }

    #[test]
    fn measures_are_a_get_with_comma_separated_keys() {
        let (instance, seen) = instance();
        // The dashboard aggregates are the one place a POST is refused: keys travel on
        // the query string, joined with commas.
        block_on(instance.measures(&["new_users", "interactions"], "2024-01-01", "2024-01-08"))
            .expect("recorded");
        let (method, url, _) = seen.borrow()[0].clone();
        assert_eq!(method, "GET", "measures are a GET");
        assert!(
            url.contains("/api/v1/admin/measures?keys=new_users%2Cinteractions"),
            "keys are one comma-separated pair, got {url}"
        );
        assert!(url.contains("start_at=2024-01-01"), "{url}");
        assert!(url.contains("end_at=2024-01-08"), "{url}");
    }

    #[test]
    fn a_filter_only_sends_the_narrowings_it_has() {
        let (instance, seen) = instance();
        let filter = ReportFilter {
            unassigned: true,
            limit: Some(20),
            ..ReportFilter::default()
        };
        block_on(instance.reports_page(&filter)).expect("recorded");
        let (method, url, _) = seen.borrow()[0].clone();
        assert_eq!(method, "GET");
        assert!(url.contains("unassigned=true"), "{url}");
        assert!(url.contains("limit=20"), "{url}");
        assert!(
            !url.contains("state="),
            "an unset state must not narrow: {url}"
        );
        assert!(!url.contains("offset="), "{url}");
    }

    #[test]
    fn a_report_is_filed_as_a_form_with_comma_separated_ids() {
        let (instance, seen) = instance();
        let mut draft = ReportDraft::new("4");
        draft.status_ids = vec!["10".to_owned(), "11".to_owned()];
        draft.comment = "spam".to_owned();
        // The fake answers a list for every route, so only the request is under test.
        let _ = block_on(instance.create_report(&draft));
        let (method, url, body) = seen.borrow()[0].clone();
        assert_eq!(method, "POST");
        assert!(
            url.ends_with("/api/v1/reports"),
            "filing is not an admin verb: {url}"
        );
        let body = body.expect("a form body");
        assert!(body.contains("account_id=4"), "{body}");
        assert!(body.contains("status_ids=10%2C11"), "{body}");
        assert!(body.contains("comment=spam"), "{body}");
    }

    #[test]
    fn assigning_nobody_sends_an_explicit_null() {
        let (instance, seen) = instance();
        let _ = block_on(instance.assign_report("9", None));
        let (method, url, body) = seen.borrow()[0].clone();
        assert_eq!(method, "POST");
        assert!(url.ends_with("/api/v1/admin/reports/9/assign"), "{url}");
        assert_eq!(body.as_deref(), Some("{\"account_id\":null}"));
    }

    #[test]
    fn a_download_is_returned_as_bytes() {
        let (instance, seen) = instance();
        let bytes = block_on(instance.download_scanner_asset("a1")).expect("recorded");
        let (method, url, body) = seen.borrow()[0].clone();
        assert_eq!(method, "GET");
        assert!(
            url.ends_with("/api/v1/admin/scanner_assets/a1/download"),
            "{url}"
        );
        assert!(body.is_none());
        // The recording transport answers `[]`; the point is that the body arrives
        // untouched rather than being decoded as JSON.
        assert_eq!(bytes, b"[]");
    }

    #[test]
    fn an_announcement_is_written_as_a_form() {
        let (instance, seen) = instance();
        let mut draft = AnnouncementDraft::new("maintenance tonight");
        draft.starts_at = Some("2024-05-01T00:00:00Z".to_owned());
        // The fake answers a list for every route, so only the request is under test.
        let _ = block_on(instance.create_announcement(draft));
        let (method, url, body) = seen.borrow()[0].clone();
        assert_eq!(method, "POST");
        assert!(url.ends_with("/api/v1/admin/announcements"), "{url}");
        let body = body.expect("a form body");
        assert!(body.contains("content=maintenance+tonight"), "{body}");
        assert!(
            body.contains("starts_at=2024-05-01T00%3A00%3A00Z"),
            "{body}"
        );
        assert!(
            !body.contains("ends_at"),
            "an unset date must be omitted: {body}"
        );
    }

    #[test]
    fn the_topology_is_read_from_the_admin_surface() {
        let (instance, seen) = instance();
        let _ = block_on(instance.topology());
        let (method, url, _) = seen.borrow()[0].clone();
        assert_eq!(method, "GET");
        assert!(
            url.starts_with("https://mastodon.example/api/v1/admin/topology"),
            "{url}"
        );
    }

    #[test]
    fn node_logs_name_the_node_in_the_path() {
        let (instance, seen) = instance();
        let _ = block_on(instance.node_logs("scan", "warn", 50));
        let (method, url, _) = seen.borrow()[0].clone();
        assert_eq!(method, "GET");
        assert!(url.contains("/api/v1/admin/topology/scan/logs?"), "{url}");
        assert!(url.contains("min_level=warn"), "{url}");
        assert!(url.contains("limit=50"), "{url}");
    }

    #[test]
    fn the_audit_log_is_read_rather_than_refused() {
        let (instance, seen) = instance();
        block_on(instance.audit(10)).expect("the audit log is served");
        let (method, url, _) = seen.borrow()[0].clone();
        assert_eq!(method, "GET");
        assert!(
            url.starts_with("https://mastodon.example/api/v1/admin/audit?limit=10"),
            "{url}"
        );
    }
}
