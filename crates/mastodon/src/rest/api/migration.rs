//! Account migration: aliases and the Move redirect, over the REST transport.

use crate::actions::MigrationActions;
use crate::model::{Account, AccountAlias, MigrationRefusal};
use crate::rest::{HttpResponse, RestInstance, Transport, decode, error_message};
use crate::{Error, Result};

/// Turns a explained refusal into the typed error, leaving every other failure alone.
///
/// The API answers a refused move or alias write with `422` and a sentence (a missing
/// alias is `404`). Recognising the sentence is what lets a shell print the reason
/// instead of showing a status code; an unrecognised body still falls through to the
/// normal status mapping, so this never claims to understand more than it does.
fn refusal(response: &HttpResponse) -> Option<Error> {
    if (200..300).contains(&response.status) {
        return None;
    }
    MigrationRefusal::parse(&error_message(&response.body)).map(Error::MigrationRefused)
}

impl<T: Transport> MigrationActions for RestInstance<T> {
    type Error = Error;

    async fn move_account(&self, acct: &str) -> Result<Account> {
        let url = self.url("/api/v1/accounts/move")?;
        let form = [("acct", acct.to_owned())];
        let response = self.transport().send(self.post_form(url, &form)).await?;
        if let Some(error) = refusal(&response) {
            return Err(error);
        }
        decode(&response)
    }

    async fn aliases(&self) -> Result<Vec<AccountAlias>> {
        let url = self.url("/api/v1/accounts/aliases")?;
        self.fetch(self.get(url)).await
    }

    async fn add_alias(&self, acct: &str) -> Result<AccountAlias> {
        let url = self.url("/api/v1/accounts/aliases")?;
        let form = [("acct", acct.to_owned())];
        let response = self.transport().send(self.post_form(url, &form)).await?;
        if let Some(error) = refusal(&response) {
            return Err(error);
        }
        decode(&response)
    }

    async fn remove_alias(&self, id: &str) -> Result<()> {
        let url = self.url_segments(&["api", "v1", "accounts", "aliases", id])?;
        let response = self.transport().send(self.delete(url)).await?;
        if let Some(error) = refusal(&response) {
            return Err(error);
        }
        crate::rest::check_status(&response)
    }
}
