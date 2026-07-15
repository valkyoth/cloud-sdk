#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use cloud_sdk_reqwest::asynchronous::{
        AsyncClientBuilder, BearerToken as AsyncBearerToken, HttpsEndpoint as AsyncHttpsEndpoint,
        RequestTimeouts as AsyncRequestTimeouts, UserAgent as AsyncUserAgent,
    };
    use cloud_sdk_reqwest::blocking::{
        BearerToken, BlockingClientBuilder, FipsTlsPolicy, HttpsEndpoint, RequestTimeouts,
        UserAgent,
    };
    use rustls::RootCertStore;
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, CertificateRevocationListDer};

    fn fips_policy() -> Option<FipsTlsPolicy> {
        let certificate = CertificateDer::from_pem_slice(include_bytes!(
            "../../../crates/cloud-sdk-reqwest/testdata/fips_root.pem"
        ))
        .ok()?;
        let crl = CertificateRevocationListDer::from_pem_slice(include_bytes!(
            "../../../crates/cloud-sdk-reqwest/testdata/fips.crl.pem"
        ))
        .ok()?;
        let mut roots = RootCertStore::empty();
        roots.add(certificate).ok()?;
        FipsTlsPolicy::new(roots, vec![crl]).ok()
    }

    #[test]
    fn hardened_client_builds_with_hickory_and_http2_unified() {
        let endpoint = HttpsEndpoint::new_custom("https://api.example.test/v1");
        let token = BearerToken::new("test-token");
        let user_agent = UserAgent::new("cloud-sdk-feature-unification-test/0.17");
        let timeouts = RequestTimeouts::new(Duration::from_secs(2), Duration::from_secs(1));
        assert!(endpoint.is_ok());
        assert!(token.is_ok());
        assert!(user_agent.is_ok());
        assert!(timeouts.is_ok());
        if let (Ok(endpoint), Ok(token), Ok(user_agent), Ok(timeouts)) =
            (endpoint, token, user_agent, timeouts)
        {
            let Some(policy) = fips_policy() else {
                return;
            };
            assert!(
                BlockingClientBuilder::new(endpoint, token, user_agent, timeouts)
                    .with_fips_tls_policy(policy)
                    .build()
                    .is_ok()
            );
        }
    }

    #[test]
    fn hardened_async_client_builds_with_hickory_and_http2_unified() {
        let runtime = tokio::runtime::Builder::new_current_thread().build();
        assert!(runtime.is_ok());
        if let Ok(runtime) = runtime {
            runtime.block_on(async {
                let endpoint = AsyncHttpsEndpoint::new_custom("https://api.example.test/v1");
                let token = AsyncBearerToken::new("test-token");
                let user_agent = AsyncUserAgent::new("cloud-sdk-feature-unification-test/0.17");
                let timeouts =
                    AsyncRequestTimeouts::new(Duration::from_secs(2), Duration::from_secs(1));
                assert!(endpoint.is_ok());
                assert!(token.is_ok());
                assert!(user_agent.is_ok());
                assert!(timeouts.is_ok());
                if let (Ok(endpoint), Ok(token), Ok(user_agent), Ok(timeouts)) =
                    (endpoint, token, user_agent, timeouts)
                {
                    assert!(
                        AsyncClientBuilder::new(endpoint, token, user_agent, timeouts)
                            .build()
                            .is_ok()
                    );
                }
            });
        }
    }
}
