use super::{
    AccountError as Error, AccountKind as Kind, AccountRecord, AccountRequest,
    AccountResponse as Response, MAX_LINKED_ACCOUNTS, MAX_OWNERS, OwnerList, PublicUser,
    request::{OwnerFilter, Selector},
    schema_table as schema,
};
use crate::{
    discovery::{DiscoveryValue as Value, value::Builder},
    endpoint::{OfficialCratesIoEndpoint, OfficialEndpointPurpose},
    identifiers::{NumericId, TeamLogin, UserLogin},
    query::{Include, Parameter},
    wire::JsonSuccess,
};
use alloc::vec::Vec;
use cloud_sdk::incremental_json::IncrementalJsonProgress;

impl AccountRequest<'_> {
    /// Consumes an admitted JSON response and clears its input on every path.
    /// Lookup identity is checked; counts/lists do not echo the requested ID/crate
    /// and rely on the bound request/response exchange for that association.
    pub fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<Response, Error> {
        if endpoint.purpose() == OfficialEndpointPurpose::StaticDownloads {
            return Err(Error::Binding);
        }
        let mut builder = Builder::default();
        let progress = success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?;
        if progress != IncrementalJsonProgress::Complete {
            return Err(Error::Json);
        }
        let mut root = builder.finish()?;
        let id = match self.0 {
            Selector::User(..) => schema::FIND_USER,
            Selector::Stats(_) => schema::GET_USER_STATS,
            Selector::Team(_) => schema::FIND_TEAM,
            Selector::Owners(_, OwnerFilter::All) => schema::LIST_OWNERS,
            Selector::Owners(_, OwnerFilter::Users) => schema::GET_USER_OWNERS,
            Selector::Owners(_, OwnerFilter::Teams) => schema::GET_TEAM_OWNERS,
        };
        crate::catalog::schema::validate_table(&root, id, 0, schema::NODES)?;
        match self.0 {
            Selector::User(login, query) => {
                let account = record(root.take("user")?, Kind::User)?;
                if !account.with_login(|s| same_user(s, login.as_str()))? {
                    return Err(Error::Binding);
                }
                let include = query.parameters().iter().any(|p| matches!(p, Parameter::Include(set) if set.values().contains(&Include::LinkedAccounts)));
                if root.get("linked_accounts")?.is_some() != include {
                    return Err(Error::Binding);
                }
                let linked = if include {
                    let links = root.take("linked_accounts")?;
                    validate_links(&links)?;
                    Some(links)
                } else {
                    None
                };
                Ok(Response::User(PublicUser { account, linked }))
            }
            Selector::Stats(_) => Ok(Response::UserStats {
                total_downloads: root.required("total_downloads")?.count(i64::MAX as u64)?,
            }),
            Selector::Team(login) => {
                let account = record(root.take("team")?, Kind::Team)?;
                // The pinned team controller compares the exact qualified login.
                if !account.with_login(|s| s == login.as_str())? {
                    return Err(Error::Binding);
                }
                Ok(Response::Team(account))
            }
            Selector::Owners(_, filter) => {
                let key = if matches!(filter, OwnerFilter::Teams) {
                    "teams"
                } else {
                    "users"
                };
                let values = root.take(key)?.into_array()?;
                if values.len() > MAX_OWNERS {
                    return Err(Error::Limit);
                }
                let mut items: Vec<AccountRecord> = Vec::new();
                items
                    .try_reserve_exact(values.len())
                    .map_err(|_| Error::Allocation)?;
                for value in values {
                    let kind = value.required("kind")?.with_text(|s| match s {
                        "user" => Ok(Kind::User),
                        "team" => Ok(Kind::Team),
                        _ => Err(Error::Value),
                    })??;
                    if matches!(
                        (filter, kind),
                        (OwnerFilter::Users, Kind::Team) | (OwnerFilter::Teams, Kind::User)
                    ) {
                        return Err(Error::Binding);
                    }
                    let item = record(value, kind)?;
                    for previous in &items {
                        if previous.kind == kind
                            && (previous.id == item.id || same_login(previous, &item)?)
                        {
                            return Err(Error::Value);
                        }
                    }
                    items.push(item);
                }
                Ok(Response::Owners(OwnerList(items)))
            }
        }
    }
}

fn record(fields: Value, kind: Kind) -> Result<AccountRecord, Error> {
    let id =
        NumericId::new(fields.required("id")?.count(i32::MAX as u64)?).map_err(|_| Error::Value)?;
    fields.required("login")?.with_text(|s| match kind {
        Kind::User => UserLogin::new(s).map(|_| ()).map_err(|_| Error::Value),
        Kind::Team => TeamLogin::new(s).map(|_| ()).map_err(|_| Error::Value),
    })??;
    for (name, maximum) in [("name", 256), ("avatar", 4096), ("url", 4096)] {
        bounded_optional_text(fields.required(name)?, maximum)?;
    }
    Ok(AccountRecord { id, kind, fields })
}
fn bounded_optional_text(value: &Value, maximum: usize) -> Result<(), Error> {
    if value.is_null() {
        return Ok(());
    }
    value.with_text(|s| {
        if s.len() > maximum {
            return Err(Error::Limit);
        }
        if s.chars().any(char::is_control) {
            return Err(Error::Value);
        }
        Ok(())
    })?
}
fn same_login(a: &AccountRecord, b: &AccountRecord) -> Result<bool, Error> {
    a.with_login(|left| {
        b.with_login(|right| match a.kind {
            Kind::User => same_user(left, right),
            Kind::Team => left.eq_ignore_ascii_case(right),
        })
    })?
}
fn same_user(a: &str, b: &str) -> bool {
    let canonical = |byte: u8| {
        if byte == b'-' {
            b'_'
        } else {
            byte.to_ascii_lowercase()
        }
    };
    a.bytes().map(canonical).eq(b.bytes().map(canonical))
}
fn validate_links(value: &Value) -> Result<(), Error> {
    let values = value.array()?;
    if values.len() > MAX_LINKED_ACCOUNTS {
        return Err(Error::Limit);
    }
    for (index, item) in values.iter().enumerate() {
        item.required("login")?
            .with_text(|s| UserLogin::new(s).map(|_| ()).map_err(|_| Error::Value))??;
        item.required("account_id")?.with_text(|s| {
            if s.is_empty() || s.len() > 256 || s.chars().any(char::is_control) {
                Err(Error::Value)
            } else {
                Ok(())
            }
        })??;
        bounded_optional_text(item.required("avatar")?, 4096)?;
        for previous in values.get(..index).ok_or(Error::Limit)? {
            let equal_id = item
                .required("account_id")?
                .with_text(|a| previous.required("account_id")?.with_text(|b| a == b))??;
            let equal_login = item.required("login")?.with_text(|a| {
                previous
                    .required("login")?
                    .with_text(|b| a.eq_ignore_ascii_case(b))
            })??;
            // The admitted linked-account schema has only the GitHub namespace.
            if equal_id || equal_login {
                return Err(Error::Value);
            }
        }
    }
    Ok(())
}

#[cfg(any(feature = "blocking", feature = "async"))]
impl crate::discovery::checked::CheckedGet for AccountRequest<'_> {
    type Response = Response;
    fn target(
        self,
        output: &mut [u8],
    ) -> Result<crate::endpoint::ApiRequestTarget<'_>, crate::query::QueryError> {
        self.write_target(output)
    }
    fn decode(
        self,
        endpoint: OfficialCratesIoEndpoint,
        success: JsonSuccess<'_>,
    ) -> Result<Response, Error> {
        self.decode(endpoint, success)
    }
}
