mod sealed {
    pub trait Sealed {}
}

/// Sealed credential kind. Applications cannot add authentication modes.
pub trait CredentialKind: sealed::Sealed {
    /// SDK-local storage bound, not a claim about upstream maximum lengths.
    const MAX_BYTES: usize;
    #[doc(hidden)]
    const KIND: u8;
}

macro_rules! kind {
    ($name:ident, $id:literal, $max:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug)]
        pub enum $name {}
        impl sealed::Sealed for $name {}
        impl CredentialKind for $name {
            const MAX_BYTES: usize = $max;
            const KIND: u8 = $id;
        }
    };
}

kind!(Api, 0, 1024, "Raw API authorization context.");
kind!(
    TrustedPublishing,
    1,
    1017,
    "Temporary Bearer authorization context."
);
kind!(
    Oidc,
    2,
    16384,
    "Compact signed OIDC assertion JSON context."
);
kind!(
    EmailConfirmation,
    3,
    512,
    "Email confirmation path context."
);
kind!(
    OwnerInvitation,
    4,
    512,
    "Owner invitation acceptance path context."
);

pub(super) fn validate<K: CredentialKind>(bytes: &[u8]) -> Result<(), super::CredentialError> {
    use super::CredentialError;
    if bytes.is_empty() {
        return Err(CredentialError::Empty);
    }
    if bytes.len() > K::MAX_BYTES {
        return Err(CredentialError::TooLong);
    }
    let valid = match K::KIND {
        0 | 1 => {
            let body = bytes.trim_ascii_end();
            let data_end = body
                .iter()
                .position(|byte| *byte == b'=')
                .unwrap_or(body.len());
            data_end > 0
                && body.get(..data_end).is_some_and(|data| {
                    data.iter()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/".contains(byte))
                })
                && body
                    .get(data_end..)
                    .is_some_and(|padding| padding.iter().all(|byte| *byte == b'='))
                && body.len() == bytes.len()
        }
        2 => {
            let mut parts = bytes.split(|byte| *byte == b'.');
            (0..3).all(|_| {
                parts.next().is_some_and(|part| {
                    !part.is_empty()
                        && part
                            .iter()
                            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(byte))
                })
            }) && parts.next().is_none()
        }
        3 | 4 => {
            bytes != b"."
                && bytes != b".."
                && bytes
                    .iter()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-._~".contains(byte))
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(CredentialError::InvalidSyntax)
    }
}
