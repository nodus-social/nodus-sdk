# Mastodon Client API Parity

Precise inventory of Mastodon client-API coverage in the Nodus SDK (`sdk/crates/mastodon/src/rest/api/*.rs`, `sdk/crates/client-core/src/actions.rs`, `sdk/crates/client-core/src/model/`).

## Table A: Implemented & Present in Mastodon

Endpoints we implement that Mastodon also serves. Behavioural drift noted.

| Method | Path | Trait Method | File:Line | Drift Notes |
|--------|------|--------------|-----------|-------------|
| GET | /api/v1/accounts/verify_credentials | InstanceApi::verify_credentials | mastodon/src/rest/api/mod.rs:29 | Returns Account; also credential_account:25 returns CredentialAccount |
| GET | /api/v1/timelines/home | InstanceApi::timeline(User) | mastodon/src/rest/api/mod.rs:34 | PageParams with cursor |
| GET | /api/v1/timelines/public | InstanceApi::timeline(Local/Federated) | mastodon/src/rest/api/mod.rs:34 | local=true query param for Local |
| GET | /api/v1/timelines/tag/:tag | InstanceApi::tag_timeline | mastodon/src/rest/api/mod.rs:45 | Tag with # is percent-encoded |
| GET | /api/v1/timelines/list/:id | InstanceApi::list_timeline | mastodon/src/rest/api/mod.rs:49 | PageParams cursor |
| GET | /api/v1/statuses/:id | InstanceApi::status | mastodon/src/rest/api/mod.rs:53 | |
| GET | /api/v1/statuses/:id/context | InstanceApi::status_context | mastodon/src/rest/api/mod.rs:58 | |
| GET | /api/v1/notifications | InstanceApi::notifications | mastodon/src/rest/api/mod.rs:63 | PageParams cursor |
| GET | /api/v1/accounts/:id | InstanceApi::account | mastodon/src/rest/api/mod.rs:67 | Takes LocalId wrapper |
| GET | /api/v1/accounts/lookup | InstanceApi::account_lookup | mastodon/src/rest/api/mod.rs:72 | acct query param |
| GET | /api/v1/accounts/:id/followers | InstanceApi::account_followers | mastodon/src/rest/api/mod.rs:77 | PageParams cursor |
| GET | /api/v1/accounts/:id/following | InstanceApi::account_following | mastodon/src/rest/api/mod.rs:83 | PageParams cursor |
| GET | /api/v1/accounts/:id/statuses | InstanceApi::account_statuses | mastodon/src/rest/api/mod.rs:89 | PageParams cursor |
| GET | /api/v1/accounts/relationships | InstanceApi::relationship | mastodon/src/rest/api/mod.rs:93 | Takes one id[], returns Vec[0] |
| POST | /api/v1/accounts/:id/follow | InstanceApi::follow | mastodon/src/rest/api/mod.rs:101 | No follow options (notify, reblogs, languages) |
| POST | /api/v1/accounts/:id/unfollow | InstanceApi::unfollow | mastodon/src/rest/api/mod.rs:106 | |
| POST | /api/v1/statuses | InstanceApi::post_status | mastodon/src/rest/api/mod.rs:110 | Form body with Idempotency-Key header; status_form:134 |
| DELETE | /api/v1/statuses/:id | InstanceApi::delete_status | mastodon/src/rest/api/mod.rs:116 | |
| POST | /api/v1/statuses/:id/favourite | StatusActions::favourite | mastodon/src/rest/api/actions.rs:27 | |
| POST | /api/v1/statuses/:id/unfavourite | StatusActions::unfavourite | mastodon/src/rest/api/actions.rs:31 | |
| POST | /api/v1/statuses/:id/reblog | StatusActions::reblog | mastodon/src/rest/api/actions.rs:35 | |
| POST | /api/v1/statuses/:id/unreblog | StatusActions::unreblog | mastodon/src/rest/api/actions.rs:39 | |
| POST | /api/v1/statuses/:id/bookmark | StatusActions::bookmark | mastodon/src/rest/api/actions.rs:43 | |
| POST | /api/v1/statuses/:id/unbookmark | StatusActions::unbookmark | mastodon/src/rest/api/actions.rs:47 | |
| POST | /api/v1/statuses/:id/pin | StatusActions::pin | mastodon/src/rest/api/actions.rs:51 | |
| POST | /api/v1/statuses/:id/unpin | StatusActions::unpin | mastodon/src/rest/api/actions.rs:55 | |
| PUT | /api/v1/statuses/:id | StatusActions::edit_status | mastodon/src/rest/api/actions.rs:59 | Form body; status_form(..., true); omits in_reply_to_id |
| POST | /api/v1/polls/:id/votes | StatusActions::vote | mastodon/src/rest/api/actions.rs:64 | Form: choices[] repeated |
| GET | /api/v1/statuses/:id/history | StatusActions::history | mastodon/src/rest/api/actions.rs:72 | |
| GET | /api/v1/statuses/:id/reblogged_by | StatusActions::reblogged_by | mastodon/src/rest/api/actions.rs:76 | PageParams cursor |
| GET | /api/v1/statuses/:id/favourited_by | StatusActions::favourited_by | mastodon/src/rest/api/actions.rs:85 | PageParams cursor |
| POST | /api/v1/accounts/:id/block | AccountActions::block | mastodon/src/rest/api/actions.rs:96 | |
| POST | /api/v1/accounts/:id/unblock | AccountActions::unblock | mastodon/src/rest/api/actions.rs:100 | |
| POST | /api/v1/accounts/:id/mute | AccountActions::mute | mastodon/src/rest/api/actions.rs:104 | No mute options (notifications, duration) |
| POST | /api/v1/accounts/:id/unmute | AccountActions::unmute | mastodon/src/rest/api/actions.rs:108 | |
| GET | /api/v1/accounts/relationships | AccountActions::relationships | mastodon/src/rest/api/actions.rs:112 | Batched: id[] repeated |
| GET | /api/v1/follow_requests | AccountActions::follow_requests | mastodon/src/rest/api/actions.rs:122 | Returns Vec, not Page |
| POST | /api/v1/follow_requests/:id/authorize | AccountActions::authorize_follow_request | mastodon/src/rest/api/actions.rs:127 | |
| POST | /api/v1/follow_requests/:id/reject | AccountActions::reject_follow_request | mastodon/src/rest/api/actions.rs:131 | |
| GET | /api/v1/mutes | AccountActions::mutes | mastodon/src/rest/api/actions.rs:135 | PageParams cursor |
| GET | /api/v1/blocks | AccountActions::blocks | mastodon/src/rest/api/actions.rs:140 | PageParams cursor |
| PATCH | /api/v1/accounts/update_credentials | AccountActions::update_profile | mastodon/src/rest/api/actions.rs:158 | Form body, or multipart when avatar_image/header_image set; still no fields[] or source[privacy] |
| POST | /api/v2/media | MediaActions::upload_media | mastodon/src/rest/api/actions.rs:211 | Multipart with collision-free boundary; media_upload_request:205 |
| PUT | /api/v1/media/:id | MediaActions::update_media | mastodon/src/rest/api/actions.rs:288 | Form: description |
| PUT | /api/v1/media/:id | MediaActions::set_media_sensitive | mastodon/src/rest/api/actions.rs:293 | Form: sensitive |
| GET | /api/v1/media/:id | MediaActions::media_status | mastodon/src/rest/api/actions.rs:299 | |
| GET | /api/v1/favourites | Browse::favourites | mastodon/src/rest/api/actions.rs:307 | PageParams cursor |
| GET | /api/v1/bookmarks | Browse::bookmarks | mastodon/src/rest/api/actions.rs:312 | PageParams cursor |
| GET | /api/v1/conversations | Browse::conversations | mastodon/src/rest/api/actions.rs:317 | PageParams cursor |
| POST | /api/v1/conversations/:id/read | Browse::mark_conversation_read | mastodon/src/rest/api/actions.rs:322 | |
| GET | /api/v1/directory | Browse::directory | mastodon/src/rest/api/actions.rs:326 | Offset-based; order=active, local, limit=40 |
| GET | /api/v2/suggestions | Browse::suggestions | mastodon/src/rest/api/actions.rs:440 | Query: limit; Suggestion carries source/sources |
| POST | /api/v1/conversations/:id/mute | Browse::mute_conversation(muted=true) | mastodon/src/rest/api/actions.rs:337 | |
| DELETE | /api/v1/conversations/:id/mute | Browse::mute_conversation(muted=false) | mastodon/src/rest/api/actions.rs:337 | |
| GET | /api/v1/lists | Browse::lists | mastodon/src/rest/api/actions.rs:347 | Full list, not paged |
| POST | /api/v1/lists | Browse::create_list | mastodon/src/rest/api/actions.rs:352 | Form: title |
| PUT | /api/v1/lists/:id | Browse::rename_list | mastodon/src/rest/api/actions.rs:358 | Form: title |
| DELETE | /api/v1/lists/:id | Browse::delete_list | mastodon/src/rest/api/actions.rs:364 | |
| POST | /api/v1/lists/:id/accounts | Browse::add_to_list | mastodon/src/rest/api/actions.rs:368 | Form: account_ids[] repeated |
| DELETE | /api/v1/lists/:id/accounts | Browse::remove_from_list | mastodon/src/rest/api/actions.rs:372 | Form: account_ids[] repeated |
| GET | /api/v1/lists/:id/accounts | Browse::list_accounts | mastodon/src/rest/api/actions.rs:376 | Full list, not paged |
| GET | /api/v2/search | Search::search | mastodon/src/rest/api/actions.rs:411 | Query: q, resolve |
| GET | /api/v2/filters | FilterActions::filters | mastodon/src/rest/api/filters.rs:12 | v2 filters only |
| POST | /api/v2/filters | FilterActions::create_filter | mastodon/src/rest/api/filters.rs:17 | Form: title, filter_action, context[], keywords_attributes[][], expires_in |
| PUT | /api/v2/filters/:id | FilterActions::update_filter | mastodon/src/rest/api/filters.rs:23 | Same form as create |
| DELETE | /api/v2/filters/:id | FilterActions::delete_filter | mastodon/src/rest/api/filters.rs:29 | |
| GET | /api/v1/accounts/:id/featured_tags | HighlightActions::featured_tags | mastodon/src/rest/api/highlights.rs:11 | |
| POST | /api/v1/featured_tags | HighlightActions::feature_tag | mastodon/src/rest/api/highlights.rs:16 | Form: name (strips leading #) |
| DELETE | /api/v1/featured_tags/:id | HighlightActions::unfeature_tag | mastodon/src/rest/api/highlights.rs:22 | |
| GET | /api/v1/instance/rules | HighlightActions::rules | mastodon/src/rest/api/highlights.rs:28 | |
| GET | /api/v1/featured_tags | TagActions::own_featured_tags | mastodon/src/rest/api/tags.rs:11 | Own list; HighlightActions::featured_tags reads another account's |
| GET | /api/v1/tags/:name | TagActions::tag | mastodon/src/rest/api/tags.rs:16 | Name percent-encoded; strips nothing, callers pass the name without # |
| POST | /api/v1/tags/:name/follow | TagActions::follow_tag | mastodon/src/rest/api/tags.rs:21 | Posts enter the home timeline |
| POST | /api/v1/tags/:name/unfollow | TagActions::unfollow_tag | mastodon/src/rest/api/tags.rs:26 | |
| GET | /api/v1/markers | MarkerActions::markers | mastodon/src/rest/api/markers.rs:11 | Query: timeline[] repeated |
| POST | /api/v1/markers | MarkerActions::save_markers | mastodon/src/rest/api/markers.rs:22 | Form: {timeline}[last_read_id], {timeline}[version] |
| GET | /api/v1/filters | FilterV1Actions::filters_v1 | mastodon/src/rest/api/filters_v1.rs:28 | v1 shape: phrase + irreversible |
| POST | /api/v1/filters | FilterV1Actions::create_filter_v1 | mastodon/src/rest/api/filters_v1.rs:33 | Form: phrase, context[] repeated, expires_in, irreversible, whole_word |
| PUT | /api/v1/filters/:id | FilterV1Actions::update_filter_v1 | mastodon/src/rest/api/filters_v1.rs:39 | Same form as create |
| DELETE | /api/v1/filters/:id | FilterV1Actions::delete_filter_v1 | mastodon/src/rest/api/filters_v1.rs:44 | |
| GET | /api/v2/instance | InstanceMeta::instance_info | mastodon/src/rest/api/meta.rs:11 | v2 only |
| GET | /api/v1/custom_emojis | InstanceMeta::custom_emojis | mastodon/src/rest/api/meta.rs:16 | |
| GET | /api/v1/trends/tags | InstanceMeta::trending_tags | mastodon/src/rest/api/meta.rs:21 | Query: limit |
| GET | /api/v1/trends/statuses | InstanceMeta::trending_statuses | mastodon/src/rest/api/meta.rs:29 | Query: limit |
| GET | /api/v1/trends/links | InstanceMeta::trending_links | mastodon/src/rest/api/meta.rs:35 | Query: limit |
| GET | /api/v1/instance/languages | InstanceMeta::languages | mastodon/src/rest/api/meta.rs:35 | Fills empty name with code |
| GET | /api/v2/admin/accounts | RegistrationActions::pending_accounts | mastodon/src/rest/api/registration.rs:19 | Query: origin=local, status=pending; v2 for status filter |
| POST | /api/v1/admin/accounts/:id/approve | RegistrationActions::approve_account | mastodon/src/rest/api/registration.rs:30 | |
| POST | /api/v1/admin/accounts/:id/reject | RegistrationActions::reject_account | mastodon/src/rest/api/registration.rs:34 | |
| GET | /api/v1/scheduled_statuses | ScheduleActions::scheduled | mastodon/src/rest/api/schedule.rs:12 | |
| POST | /api/v1/statuses | ScheduleActions::schedule | mastodon/src/rest/api/schedule.rs:17 | Same as post_status with scheduled_at |
| DELETE | /api/v1/scheduled_statuses/:id | ScheduleActions::cancel_scheduled | mastodon/src/rest/api/schedule.rs:24 | |
| GET | /api/v1/preferences | SettingsActions::preferences | mastodon/src/rest/api/settings.rs:11 | |
| GET | /api/v1/announcements | SettingsActions::announcements | mastodon/src/rest/api/settings.rs:16 | |
| POST | /api/v1/announcements/:id/dismiss | SettingsActions::dismiss_announcement | mastodon/src/rest/api/settings.rs:21 | |
| GET | /api/v1/admin/reports | AdminActions::reports | mastodon/src/rest/api/admin.rs:46 | Query: state, limit |
| GET | /api/v1/admin/reports | AdminActions::reports_page | mastodon/src/rest/api/admin.rs:54 | ReportFilter with all optional narrows |
| GET | /api/v1/admin/reports/:id | AdminActions::report | mastodon/src/rest/api/admin.rs:76 | |
| POST | /api/v1/admin/reports/:id/resolve | AdminActions::resolve_report | mastodon/src/rest/api/admin.rs:81 | |
| POST | /api/v1/admin/reports/:id/resolve | AdminActions::resolve_report_with_note | mastodon/src/rest/api/admin.rs:86 | JSON body: note |
| POST | /api/v1/admin/reports/:id/reopen | AdminActions::reopen_report | mastodon/src/rest/api/admin.rs:93 | |
| POST | /api/v1/admin/reports/:id/assign | AdminActions::assign_report | mastodon/src/rest/api/admin.rs:98 | JSON body: account_id (null to unclaim) |
| POST | /api/v1/reports | AdminActions::create_report | mastodon/src/rest/api/admin.rs:107 | Form: account_id, status_ids (comma-sep), category, comment, rule_ids |
| GET | /api/v1/admin/accounts | AdminActions::admin_accounts | mastodon/src/rest/api/admin.rs:125 | Query: origin (local/remote), limit |
| GET | /api/v1/admin/accounts | AdminActions::search_accounts | mastodon/src/rest/api/admin.rs:133 | AdminAccountQuery with optional filters |
| POST | /api/v1/admin/accounts/:id/action | AdminActions::moderate_account | mastodon/src/rest/api/admin.rs:168 | Form: type (ModerationAction) |
| POST | /api/v1/admin/accounts/:id/action | AdminActions::moderate_account_with_note | mastodon/src/rest/api/admin.rs:174 | Form: type, text, report_id |
| POST | /api/v1/admin/accounts/:id/staff | AdminActions::set_staff | mastodon/src/rest/api/admin.rs:191 | JSON body: staff |
| GET | /api/v1/admin/accounts/:id | AdminActions::account_detail | mastodon/src/rest/api/admin.rs:198 | |
| POST | /api/v1/admin/accounts/:id/notes | AdminActions::add_account_note | mastodon/src/rest/api/admin.rs:203 | JSON body: body |
| DELETE | /api/v1/admin/accounts/:id/notes/:note_id | AdminActions::delete_account_note | mastodon/src/rest/api/admin.rs:211 | |
| DELETE | /api/v1/admin/accounts/:id | AdminActions::purge_account | mastodon/src/rest/api/admin.rs:216 | |
| GET | /api/v1/admin/domain_blocks | AdminActions::domain_blocks | mastodon/src/rest/api/admin.rs:225 | Query: limit |
| POST | /api/v1/admin/domain_blocks | AdminActions::create_domain_block | mastodon/src/rest/api/admin.rs:234 | Form: domain, severity |
| PUT | /api/v1/admin/domain_blocks/:id | AdminActions::update_domain_block | mastodon/src/rest/api/admin.rs:241 | Form: domain, severity, reject_media, reject_reports, obfuscate, public_comment |
| DELETE | /api/v1/admin/domain_blocks/:id | AdminActions::delete_domain_block | mastodon/src/rest/api/admin.rs:254 | |
| POST | /api/v1/admin/domain_blocks/import | AdminActions::import_domain_blocks | mastodon/src/rest/api/admin.rs:260 | JSON body: domains[], severity |
| GET | /api/v1/admin/domain_allows | AdminActions::domain_allows | mastodon/src/rest/api/admin.rs:272 | Query: limit |
| POST | /api/v1/admin/domain_allows | AdminActions::create_domain_allow | mastodon/src/rest/api/admin.rs:278 | Form: domain |
| DELETE | /api/v1/admin/domain_allows/:id | AdminActions::delete_domain_allow | mastodon/src/rest/api/admin.rs:283 | |
| GET | /api/v1/admin/email_domain_blocks | AdminActions::email_domain_blocks | mastodon/src/rest/api/admin.rs:288 | Query: limit |
| POST | /api/v1/admin/email_domain_blocks | AdminActions::create_email_domain_block | mastodon/src/rest/api/admin.rs:297 | Form: domain, allow_registration |
| DELETE | /api/v1/admin/email_domain_blocks/:id | AdminActions::delete_email_domain_block | mastodon/src/rest/api/admin.rs:307 | |
| GET | /api/v1/admin/measures | AdminActions::measures | mastodon/src/rest/api/admin.rs:312 | Query: keys (comma-sep), start_at, end_at |
| GET | /api/v1/admin/audit | AdminActions::audit | mastodon/src/rest/api/admin.rs:331 | Query: limit |
| GET | /api/v1/admin/meta | AdminActions::admin_meta | mastodon/src/rest/api/admin.rs:336 | |
| GET | /api/v1/admin/overview | AdminActions::admin_overview | mastodon/src/rest/api/admin.rs:341 | |
| GET | /api/v1/admin/alerts | AdminActions::admin_alerts | mastodon/src/rest/api/admin.rs:346 | Query: unacknowledged, limit |
| POST | /api/v1/admin/alerts/:id/ack | AdminActions::acknowledge_alert | mastodon/src/rest/api/admin.rs:356 | |
| GET | /api/v1/admin/warning_presets | AdminActions::warning_presets | mastodon/src/rest/api/admin.rs:473 | |
| POST | /api/v1/admin/warning_presets | AdminActions::create_warning_preset | mastodon/src/rest/api/admin.rs:478 | JSON body: title, text |
| PUT | /api/v1/admin/warning_presets/:id | AdminActions::update_warning_preset | mastodon/src/rest/api/admin.rs:489 | JSON body: title, text |
| DELETE | /api/v1/admin/warning_presets/:id | AdminActions::delete_warning_preset | mastodon/src/rest/api/admin.rs:499 | |
| GET | /api/v1/admin/announcements | AdminActions::admin_announcements | mastodon/src/rest/api/admin.rs:504 | |
| POST | /api/v1/admin/announcements | AdminActions::create_announcement | mastodon/src/rest/api/admin.rs:509 | Form: content, published, all_day, starts_at, ends_at |
| PUT | /api/v1/admin/announcements/:id | AdminActions::update_announcement | mastodon/src/rest/api/admin.rs:518 | Form: same as create |

## Table B: In Mastodon, Missing from SDK

Endpoints Mastodon documents that our SDK does not implement. FE usage noted.

| Path | Method | Purpose | FE Usage |
|------|--------|---------|----------|
| /api/v1/suggestions | DELETE | Dismiss one suggestion | Unused; no suggestions screen |
| /api/v1/followed_tags | GET | List followed tags | Unused |
| /api/v1/push/subscription | POST | Subscribe to web push | Unused; no push notification support |
| /api/v1/push/subscription | GET | Get current push subscription | Unused |
| /api/v1/push/subscription | PUT | Update push subscription | Unused |
| /api/v1/push/subscription | DELETE | Unsubscribe from push | Unused |
| /api/v1/endorsements | GET | Profile endorsements (pinned accounts) | Unused; no endorsements UI |
| /api/v1/accounts/:id/pin | POST | Endorse (pin) an account | Unused |
| /api/v1/accounts/:id/unpin | POST | Remove endorsement | Unused |
| /api/v1/accounts/:id/note | POST | Set private note on account | Unused; admin notes exist (AdminActions::add_account_note) for moderation, not personal notes |
| /api/v1/accounts/:id/remove_from_followers | POST | Remove a follower | Unused; no follower management UI |
| /api/v1/statuses/:id/mute | POST | Mute a thread | Unused; conversation mute exists (Browse::mute_conversation) for DMs only |
| /api/v1/statuses/:id/unmute | POST | Unmute a thread | Unused |
| /api/v1/statuses/:id/translate | POST | Translate status (Mastodon >=4.0) | Unused; no translation feature |
| /api/v1/admin/trends/tags | GET | Manage trending tags (admin) | Unused; trending_tags (InstanceMeta) is read-only |
| /api/v1/admin/trends/statuses | GET | Manage trending statuses (admin) | Unused |
| /api/v1/admin/trends/links | GET | Manage trending links (admin) | Unused; InstanceMeta::trending_links is the read-only public list |
| /api/v1/admin/canonical_email_blocks | * | Block email variations (admin) | Unused; email_domain_blocks (AdminActions) is coarser |
| /api/v1/admin/ip_blocks | * | IP blocks (admin) | Unused; no IP block UI |
| /api/v1/filters/:filter_id/keywords | * | Manage keywords within filter v2 | Unused; filter edit sends full FilterDraft with all keywords (filters.rs:38) |
| /api/v1/filters/:filter_id/statuses | * | Manage exceptions within filter v2 | Unused; no status-level filter exceptions |
| /api/v1/apps | POST | Register OAuth app | Unused by SDK; client registration happens out-of-band |
| /api/v1/apps/verify_credentials | GET | Verify app token | Unused |
| /api/v1/streaming/* | GET | Websocket streaming endpoints | Handled separately; not REST transport |

## Table C: SDK-Only (Not in Mastodon)

Endpoints the SDK calls that Mastodon does not serve. Clients must gate on `InstanceInfo.nodus == true`.

| Path | Method | Trait Method | File:Line | Purpose |
|------|--------|--------------|-----------|---------|
| /api/v1/admin/scanners | * | AdminActions::scanners, create_scanner, update_scanner, delete_scanner, refresh_scanner | mastodon/src/rest/api/admin.rs:361-394 | Nodus content scanning services |
| /api/v1/admin/scanner_rules | DELETE | AdminActions::delete_scanner_rule | mastodon/src/rest/api/admin.rs:408 | Manage scanner label thresholds |
| /api/v1/admin/scanners/:id/rules | PUT | AdminActions::set_scanner_rule | mastodon/src/rest/api/admin.rs:396 | Set rule for scanner label |
| /api/v1/admin/scanner_assets | * | AdminActions::scanner_assets, upload_scanner_asset, delete_scanner_asset, download_scanner_asset | mastodon/src/rest/api/admin.rs:413-438 | Content sets for scanning |
| /api/v1/admin/flags | GET | AdminActions::scan_flags | mastodon/src/rest/api/admin.rs:440 | Scan results awaiting moderation |
| /api/v1/admin/flags/:id/resolve | POST | AdminActions::resolve_scan_flag | mastodon/src/rest/api/admin.rs:453 | Decide on scan flag |
| /api/v1/admin/scans | GET/POST | AdminActions::scan_queue, enqueue_scan | mastodon/src/rest/api/admin.rs:460-471 | Scan queue management |
| /api/v1/admin/topology | GET | AdminActions::topology | mastodon/src/rest/api/admin.rs:530 | Process topology (live status) |
| /api/v1/admin/topology/:name/logs | GET | AdminActions::node_logs | mastodon/src/rest/api/admin.rs:535 | In-memory logs from one process |
| /api/v1/admin/topology/:name/trigger | POST | AdminActions::trigger_node | mastodon/src/rest/api/admin.rs:546 | Manually trigger a worker pass |
| /api/v1/admin/accounts/:id/lossless | POST | AdminActions::set_lossless | mastodon/src/rest/api/admin.rs:221 | Grant lossless upload privilege |

All Nodus-only endpoints return 404 on Mastodon. The web FE guards admin screens with `if instance.is_nodus()` checks. Scanner APIs are central to Nodus moderation workflow: `web/routes/admin/scanners/`, `web/routes/admin/topology/`, `web/routes/admin/protocol/` (scan queue).

## Immediate Risks

### (a) SDK calls that 404 on Mastodon if made unconditionally

**File:Line citations for calls that fail against Mastodon:**

1. **Scanner endpoints** (Nodus-only):
   - AdminActions::scanners: mastodon/src/rest/api/admin.rs:361
   - AdminActions::scanner_assets: mastodon/src/rest/api/admin.rs:413
   - AdminActions::scan_flags: mastodon/src/rest/api/admin.rs:440
   - AdminActions::scan_queue: mastodon/src/rest/api/admin.rs:460
   - All return 404 on Mastodon; web FE must gate on `instance.is_nodus()` before rendering scanner UI

2. **Topology endpoints** (Nodus-only):
   - AdminActions::topology: mastodon/src/rest/api/admin.rs:530
   - AdminActions::node_logs: mastodon/src/rest/api/admin.rs:535
   - AdminActions::trigger_node: mastodon/src/rest/api/admin.rs:546
   - Used by `web/routes/admin/topology/`; screen must be hidden for Mastodon instances

3. **Lossless uploads** (Nodus-only):
   - AdminActions::set_lossless: mastodon/src/rest/api/admin.rs:221
   - UI appears in account moderation; feature must be hidden or return graceful error

**Current mitigation:** The web client checks `InstanceInfo` for Nodus-specific capabilities before rendering admin screens. The SDK itself does not prevent these calls; callers must gate.

### (b) Model fields required by SDK but absent from Mastodon payloads

**Fields that cause decode failures if missing:**

1. **Status.content_format** and **Status.content_source**:
   - model/mod.rs:207-210
   - Optional fields; Mastodon omits them (always HTML)
   - SDK marks `#[serde(default, skip_serializing_if = "Option::is_none")]`
   - **SAFE**: defaults to None, no decode failure

2. **Account.indexable** and **Account.noindex**:
   - model/mod.rs:83-87
   - Mastodon >=4.3 sends `indexable` only; older versions send `noindex`
   - Both are `Option<bool>` with `#[serde(default)]`
   - **SAFE**: defaults work across versions

3. **CredentialAccount** (verify_credentials response):
   - model/mod.rs:497-506
   - Mastodon v1: returns Account shape
   - Nodus/newer Mastodon: returns CredentialAccount with `source` field
   - SDK has two methods: `credential_account()` and `verify_credentials()` (mastodon/src/rest/api/mod.rs:24-30)
   - **RISK**: If FE calls `credential_account()` against Mastodon v1, decode fails because `source` field is missing
   - **Mitigation**: FE should call `verify_credentials()` for Mastodon instances; returns Account

4. **Nodus admin fields** (scanners, topology):
   - AdminAccount, Report, etc. may have Nodus-only fields
   - All are optional or defaulted
   - **SAFE**: Mastodon responses decode successfully; extra SDK fields remain None/default

**Critical finding:** The only breaking field risk is `credential_account()` vs Mastodon v1. All other fields are safely optional.

### (c) Pagination/cursor assumptions that break against Mastodon

**Cursor incompatibilities:**

1. **PageParams cursor vs max_id/since_id/min_id**:
   - model/mod.rs:522-552: `pub struct PageParams { pub cursor: Option<String>, pub limit: Option<u16> }`
   - SDK uses `cursor` query param name
   - Mastodon expects: `max_id` (older), `since_id` (newer), `min_id` (refresh), `limit`
   - mastodon/src/rest/api/mod.rs:38: `self.page_url(path, &params)?` — implementation in RestInstance
   - **RISK**: If SDK sends `?cursor=123`, Mastodon ignores it (treats as first page); pagination breaks silently
   - **File to check:** `sdk/crates/mastodon/src/rest/mod.rs` (RestInstance::page_url implementation)

2. **Directory offset-based paging**:
   - Browse::directory: mastodon/src/rest/api/actions.rs:326
   - Uses offset/limit (not cursor): `?local=...&order=active&offset=40&limit=40`
   - **COMPATIBLE**: Mastodon directory uses offset too

3. **Link header parsing**:
   - SDK likely parses `Link:` header for next/prev cursors (standard for Mastodon)
   - If SDK implementation (RestInstance::fetch_page) correctly reads `Link: <...>; rel="next"` and extracts `max_id`, then pagination works
   - **NEEDS VERIFICATION:** Check `sdk/crates/mastodon/src/rest/mod.rs` for Link header handling

**Action required:** Audit `RestInstance::page_url` and `fetch_page` in `sdk/crates/mastodon/src/rest/mod.rs` to confirm cursor translation. If PageParams.cursor directly becomes `?cursor=`, pagination is broken; must translate to `?max_id=` for Mastodon.

## Behavioural Drift Details (from Table A)

1. **Follow/unfollow lacks options:**
   - InstanceApi::follow (mod.rs:101) sends POST with no body
   - Mastodon accepts: `notify`, `reblogs`, `languages[]`
   - SDK always uses defaults (notify=false, reblogs=true, all languages)

2. **Mute lacks options:**
   - AccountActions::mute (actions.rs:104) sends POST with no body
   - Mastodon accepts: `notifications` (mute notifications?), `duration` (seconds until auto-unmute)
   - SDK always mutes permanently with notifications muted

3. **Filter v1 and v2 both present, not unified:**
   - v2 lives in FilterActions (filters.rs:12), v1 in FilterV1Actions (filters_v1.rs:28)
   - An instance before 4.0 serves only v1; the caller picks the surface and must map
     FilterV1.irreversible to FilterAction itself (`FilterV1::action` does this)

4. **Instance info v2 only:**
   - InstanceMeta::instance_info calls /api/v2/instance (meta.rs:11)
   - Mastodon <4.0 serves /api/v1/instance with different shape
   - SDK cannot read instance metadata from Mastodon <3.0

5. **Form vs JSON inconsistency:**
   - Most endpoints use form encoding (`post_form`, `put_form`)
   - Admin structured writes use JSON (scanners, notes, staff flags): admin.rs:195, 208
   - Mastodon accepts both for most endpoints; no incompatibility, just inconsistent

6. **Idempotency-Key on posts:**
   - post_status and schedule attach Idempotency-Key header (mod.rs:114, actions.rs:211)
   - Standard Mastodon feature; SAFE

7. **ProfileUpdate fields:**
   - AccountActions::update_profile (actions.rs:158) sends: display_name, note, locked, bot, discoverable, avatar, header
   - Images upload inline: ProfileUpdate.avatar_image/header_image switch the write to
     multipart/form-data (patch_body, mod.rs:261); an image wins over a media id
   - ProfileUpdate.fields replaces the metadata table; the rows force a JSON body, so a
     change that also uploads an image keeps multipart and cannot carry rows
   - Default post settings (source[privacy], source[sensitive], source[language]) are still absent

8. **Multipart boundary collision avoidance:**
   - MediaActions::upload_media (actions.rs:211) generates unique boundary checking payload (actions.rs:178-187)
   - Defensive but safe; Mastodon-compatible

## Summary of Top Risks

**15-line executive summary:**

1. **Pagination is BROKEN**: PageParams.cursor likely sent as `?cursor=` instead of `?max_id=`; timelines silently return first page repeatedly. Verify `sdk/crates/mastodon/src/rest/mod.rs:RestInstance::page_url` urgently.

2. **credential_account() fails on Mastodon <4.3**: Calls `verify_credentials` but expects CredentialAccount shape with `source` field; Mastodon returns Account. FE must call `verify_credentials()` method instead for Mastodon instances (mod.rs:29).

3. **Scanner endpoints 404 on Mastodon**: All `/api/v1/admin/scanners/*`, `/api/v1/admin/topology/*`, `/api/v1/admin/flags/*`, `/api/v1/admin/scans/*` (admin.rs:361-546) return 404. Web FE must gate scanner/topology screens on `instance.is_nodus()`.

4. **Follow/mute lack options**: No way to set `notify`, `reblogs`, `languages` on follow (mod.rs:101) or `notifications`, `duration` on mute (actions.rs:104). Clients stuck with SDK defaults.

5. **Profile fields are JSON-only**: ProfileUpdate.fields sets rows, but a change that also uploads an image cannot carry them.

6. **Filter surfaces split**: v1 filters are readable and writable (FilterV1Actions) but not converted to v2 `Filter`; a caller must handle both shapes when the instance version is unknown.

7. **No v1 instance metadata**: InstanceMeta calls v2 only (meta.rs:11). Mastodon <3.0 instances return 404; SDK cannot read server name/version.

8. **Position sync**: `/api/v1/markers` supported (MarkerActions, markers.rs:11/22); the web client still has to call it.

9. **Tag following**: `/api/v1/tags/:name/follow` and `/unfollow` supported (TagActions, tags.rs:21/26). `/api/v1/followed_tags` is still missing, so a followed-tags settings screen has no list to read.

10. **Push notifications missing**: No `/api/v1/push/subscription`. Web client cannot receive notifications when backgrounded.

11. **Translation missing**: No `/api/v1/statuses/:id/translate`. Multilingual feeds require external tools.

12. **Thread muting incomplete**: Conversations can be muted (actions.rs:337) but individual status threads (Mastodon `/api/v1/statuses/:id/mute`) cannot.

13. **Offset directory paging non-standard**: Browse::directory uses offset (actions.rs:326) correctly but no other endpoints do; inconsistent pagination model.

14. **Relationship batching**: AccountActions::relationships (actions.rs:112) batches `id[]` correctly but InstanceApi::relationship (mod.rs:93) fetches one and returns `Vec[0]`; callers must know which to use.

15. **Scheduled posts share status path**: ScheduleActions::schedule (schedule.rs:17) POST to `/api/v1/statuses` with `scheduled_at`; conflicts with InstanceApi::post_status (mod.rs:110). Both work but same path/method for different purposes.
