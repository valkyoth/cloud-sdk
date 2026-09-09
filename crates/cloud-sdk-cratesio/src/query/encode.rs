use super::{ApiPath, MAX_TARGET_BYTES, Parameter, Query, QueryError};
use crate::endpoint::ApiRequestTarget;
use cloud_sdk::buffer::{SnapshotEncoder, encode_snapshot_bounded};

impl Query<'_> {
    /// Writes query text without `?`, atomically, in stable key order.
    pub fn write(self, output: &mut [u8]) -> Result<&str, QueryError> {
        let len =
            encode_snapshot_bounded(self, output, MAX_TARGET_BYTES, QueryError::Output, encode)?;
        core::str::from_utf8(output.get(..len).ok_or(QueryError::Output)?)
            .map_err(|_| QueryError::Output)
    }
    /// Writes a complete target after binding query policy to the typed path.
    pub fn write_target<'o>(
        self,
        path: ApiPath<'_>,
        output: &'o mut [u8],
    ) -> Result<ApiRequestTarget<'o>, QueryError> {
        if !path.supports(self.operation) {
            return Err(QueryError::Operation);
        }
        let len = encode_snapshot_bounded(
            (path, self),
            output,
            MAX_TARGET_BYTES,
            QueryError::Output,
            |(path, query), encoder| {
                path.encode(encoder)?;
                if !query.parameters.is_empty() {
                    encoder.byte(b'?')?;
                    encode(query, encoder)?;
                }
                Ok(())
            },
        )?;
        let text = core::str::from_utf8(output.get(..len).ok_or(QueryError::Output)?)
            .map_err(|_| QueryError::Output)?;
        ApiRequestTarget::new(text).map_err(|_| QueryError::Syntax)
    }
}

fn encode(
    query: Query<'_>,
    encoder: &mut SnapshotEncoder<'_, QueryError>,
) -> Result<(), QueryError> {
    let mut first = true;
    let mut after = "";
    // The bounded immutable slice is sorted without allocation or mutation.
    for _ in 0..query.parameters.len() {
        let next = query
            .parameters
            .iter()
            .filter(|p| p.key() > after)
            .min_by_key(|p| p.key())
            .ok_or(QueryError::Duplicate)?;
        after = next.key();
        encode_parameter(*next, &mut first, encoder)?;
    }
    Ok(())
}

fn encode_parameter(
    p: Parameter<'_>,
    first: &mut bool,
    e: &mut SnapshotEncoder<'_, QueryError>,
) -> Result<(), QueryError> {
    use Parameter as P;
    let key = p.key();
    match p {
        P::Search(v) => e.query_pair(first, key, v.as_str()),
        P::Category(v) => e.query_pair(first, key, v.as_str()),
        P::Keyword(v) => e.query_pair(first, key, v.as_str()),
        P::Crate(v) => e.query_pair(first, key, v.as_str()),
        P::Seek(v) => e.query_pair(first, key, v.as_str()),
        P::BeforeDate(v) => e.query_pair(first, key, v.as_str()),
        P::Sort(v) => e.query_pair(first, key, v.as_str()),
        P::Page(v) => e.query_u64(first, key, u64::from(v.get())),
        P::PerPage(v) => e.query_u64(first, key, u64::from(v.get())),
        P::UserId(v) | P::TeamId(v) => e.query_u64(first, key, u64::from(v.get())),
        P::Following | P::IncludeYanked => e.query_pair(first, key, "yes"),
        P::Ids(values) => {
            for v in values {
                e.query_pair(first, key, v.as_str())?;
            }
            Ok(())
        }
        P::Versions(values) => {
            for v in values {
                e.query_pair(first, key, v.as_str())?;
            }
            Ok(())
        }
        P::Letter(letter) => {
            let mut b = [0; 4];
            e.query_pair(first, key, letter.encode_utf8(&mut b))
        }
        P::Include(values) => {
            e.query_separator(first)?;
            e.string("include=")?;
            for (i, value) in values.values().iter().enumerate() {
                if i != 0 {
                    e.string("%2C")?;
                }
                e.percent_encoded(value.as_str())?;
            }
            Ok(())
        }
        P::AllKeywords(values) => {
            e.query_separator(first)?;
            e.string("all_keywords=")?;
            for (i, value) in values.iter().enumerate() {
                if i != 0 {
                    e.string("%20")?;
                }
                e.percent_encoded(value.as_str())?;
            }
            Ok(())
        }
    }
}
