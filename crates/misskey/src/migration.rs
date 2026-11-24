//! Account migration on Misskey.
//!
//! Misskey has no alias endpoints. It publishes aliases as `alsoKnownAs` on the
//! signed-in user (`i`) and accepts them as a replacement list on `i/update`; a move is
//! `i/move` with `moveToAccount`. Each listed alias is stored as an account id, so
//! listing resolves every id back through `users/show` to give the caller account
//! documents rather than bare ids.

use nodus_client_core::actions::MigrationActions;
use nodus_client_core::model::{Account, AccountAlias, MigrationRefusal};
use nodus_client_core::rest::Transport;
use nodus_client_core::{Error, Result};
use serde_json::{Value, json};

use crate::{MisskeyInstance, convert};

impl<T: Transport> MigrationActions for MisskeyInstance<T> {
    type Error = Error;

    async fn move_account(&self, acct: &str) -> Result<Account> {
        self.call("/api/i/move", json!({ "moveToAccount": acct }))
            .await?;
        let me = self.call("/api/i", json!({})).await?;
        convert::account(&me).ok_or_else(|| Error::Decode("i is not an account".to_owned()))
    }

    async fn aliases(&self) -> Result<Vec<AccountAlias>> {
        let ids = self.alias_ids().await?;
        let mut aliases = Vec::with_capacity(ids.len());
        for id in ids {
            match user_by_body(self, json!({ "userId": id })).await {
                Ok(account) => aliases.push(AccountAlias {
                    id: account.id.clone(),
                    accounts: vec![account],
                }),
                // The id survived in `alsoKnownAs` after its account stopped resolving.
                // The list still has to render, so keep the id and say there is no
                // document for it rather than dropping an alias the profile publishes.
                Err(Error::Status { .. }) => aliases.push(AccountAlias {
                    id,
                    accounts: Vec::new(),
                }),
                Err(error) => return Err(error),
            }
        }
        Ok(aliases)
    }

    async fn add_alias(&self, acct: &str) -> Result<AccountAlias> {
        let ids = self.alias_ids().await?;
        let target = user_by_body(self, user_body(acct)).await?;
        if ids.contains(&target.id) {
            return Err(Error::MigrationRefused(
                MigrationRefusal::AliasAlreadyListed,
            ));
        }
        let mut handles = Vec::with_capacity(ids.len() + 1);
        for id in &ids {
            handles.push(user_by_body(self, json!({ "userId": id })).await?.acct);
        }
        handles.push(acct.to_owned());
        self.call("/api/i/update", json!({ "alsoKnownAs": handles }))
            .await?;
        Ok(AccountAlias {
            id: target.id.clone(),
            accounts: vec![target],
        })
    }

    async fn remove_alias(&self, id: &str) -> Result<()> {
        let ids = self.alias_ids().await?;
        if !ids.iter().any(|existing| existing == id) {
            return Err(Error::MigrationRefused(MigrationRefusal::AliasNotFound));
        }
        let mut handles = Vec::with_capacity(ids.len().saturating_sub(1));
        for existing in ids.iter().filter(|existing| existing.as_str() != id) {
            handles.push(
                user_by_body(self, json!({ "userId": existing }))
                    .await?
                    .acct,
            );
        }
        self.call("/api/i/update", json!({ "alsoKnownAs": handles }))
            .await?;
        Ok(())
    }
}

impl<T: Transport> MisskeyInstance<T> {
    /// The signed-in user's alias ids, as `i` reports them.
    async fn alias_ids(&self) -> Result<Vec<String>> {
        let me = self.call("/api/i", json!({})).await?;
        Ok(me
            .get("alsoKnownAs")
            .and_then(Value::as_array)
            .map(|ids| {
                ids.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default())
    }
}

/// Resolve one user through `users/show`.
async fn user_by_body<T: Transport>(client: &MisskeyInstance<T>, body: Value) -> Result<Account> {
    let value = client.call("/api/users/show", body).await?;
    convert::account(&value).ok_or_else(|| Error::Decode("user is not an account".to_owned()))
}

/// `users/show` takes a local username without a host, and `username` + `host` otherwise.
fn user_body(acct: &str) -> Value {
    let acct = acct.strip_prefix('@').unwrap_or(acct);
    match acct.split_once('@') {
        Some((username, host)) => json!({ "username": username, "host": host }),
        None => json!({ "username": acct }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_alias_handle_splits_into_username_and_host() {
        assert_eq!(user_body("alice"), json!({ "username": "alice" }));
        assert_eq!(
            user_body("@alice@remote.test"),
            json!({ "username": "alice", "host": "remote.test" })
        );
    }
}
