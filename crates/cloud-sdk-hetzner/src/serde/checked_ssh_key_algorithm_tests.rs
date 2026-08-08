//! Full checked-decoder coverage for every admitted SSH public-key algorithm.

use cloud_sdk::transport::StatusCode;

use super::checked_fixtures::resource_value;
use super::checked_test_support::{decode_response, prepared, response};
use super::{HetznerSuccess, SecurityResource};
use crate::SECURITY_SERVICE_ID;

const ED25519: &str = concat!(
    "ssh-ed25519 ",
    "AAAAC3NzaC1lZDI1NTE5AAAAILM+rvN+ot98qgEN796jTiQfZfG1KaT0PtFDJ/XFSqti ",
    "user@example.com"
);
const ECDSA_P256: &str = concat!(
    "ecdsa-sha2-nistp256 ",
    "AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBHwf2HMM5TRXvo2S",
    "QJjsNkiDD5KqiiNjrGVv3UUh+mMT5RHxiRtOnlqvjhQtBq0VpmpCV/PwUdhOig4vkbqAcEc= ",
    "user@example.com"
);
const ECDSA_P384: &str = concat!(
    "ecdsa-sha2-nistp384 ",
    "AAAAE2VjZHNhLXNoYTItbmlzdHAzODQAAAAIbmlzdHAzODQAAABhBC5ugtxUB/EEoREX",
    "x8BbGZPDzrPbJfrmi6FpUCpP+TldmtNrVD6AFP8V1wjiHwn1hapt+tV1t5uUNBi4YZjZvN",
    "mwf/+TmbFdQ9NO+usuVrezPP+ICyQrPgtYr5bHWEHsQQ== user@example.com"
);
const ECDSA_P521: &str = concat!(
    "ecdsa-sha2-nistp521 ",
    "AAAAE2VjZHNhLXNoYTItbmlzdHA1MjEAAAAIbmlzdHA1MjEAAACFBAFhNpNPGSsj2WH79E",
    "yBhBZgAs6ix9GLIK0BjQRu8GjT6CUP1OnxfKZpOoVUwyaabZ9XYqL5osuHl9SyAd5CHT3MW",
    "AEDy5R6hYu3eD34Y/gpUdlvkaeSXX4rqtJuR+Py+lsHyCcoSKRCO3UNetK4tpLWbd7K7FO",
    "FCGsf0baCyikciNY3Yg== user@example.com"
);
const SK_ED25519: &str = concat!(
    "sk-ssh-ed25519@openssh.com ",
    "AAAAGnNrLXNzaC1lZDI1NTE5QG9wZW5zc2guY29tAAAAICFo/k5LU8863u66YC9eUO2170Q",
    "duohPURkQnbLa/dczAAAABHNzaDo= user@example.com"
);
const SK_ECDSA_P256: &str = concat!(
    "sk-ecdsa-sha2-nistp256@openssh.com ",
    "AAAAInNrLWVjZHNhLXNoYTItbmlzdHAyNTZAb3BlbnNzaC5jb20AAAAIbmlzdHAyNTYAAABB",
    "BIELQJ2DgvaX1yQlKFokfWM2suuaCFI2qp0eJodHyg6O4ifxc3XpRKd1OS8dNYQtE/YjdXS",
    "rA+AOnMF5ns2Nkx4AAAAEc3NoOg== user@example.com"
);
const RSA: &str = concat!(
    "ssh-rsa ",
    "AAAAB3NzaC1yc2EAAAADAQABAAABgQCmjkeMm8k3JkNrf16eb5pG4bc77B6Mt3VN4salts",
    "RV8vASpyWa/PlBgdaeldOaNJ5NK0gqU3KyiUNzHbdcc8572e7IUBDJS/rlaWARiSL4aos2V",
    "bNX0k56Z5zYp9m/bq5m9/mlb+PQkNBjIhimgpYNiq2TwBiYeA6tLb79cPtHA0cX5BLk/a5",
    "oUpLsiR4kI/f+Q98vVDKasKXXVh5YLkLobrruDB6er2A9fOcIUF0O4JCRLh/Dc161gE3fQ",
    "rYTMQenbppZzfxrZfQ8YwLPvKjnqm+XRX+pbTtaJuj0EgTSzUK+EZxoSw8CNwiZpxrjwec",
    "TMVQ8w/srQmh4ABGuTqk0wP8HcI7hg+fpBv7kiejh5X/Oehxt+Puu85u9GVXb1a0av/vhJ",
    "vUCBcuISvCA/z1wVJ0xdLhb1/ZiTDdTzyNbZQ0OQijzK+e1SlkNhp+3eGVZu3pNZvnTppw",
    "IXv3wg6kV1HodkWGgh1ayY7Buc52Z8okDYqvJat5CzOj5OaQNr/k= user@example.com"
);

#[test]
fn every_admitted_ssh_algorithm_decodes_through_the_checked_response_path() {
    for (public_key, fingerprint) in [
        (ED25519, "ae:6f:ba:1b:70:2c:ae:c7:5c:ab:6e:4d:5e:d4:c7:23"),
        (RSA, "70:d6:f8:c5:4b:5f:cd:88:1d:6c:21:5d:8e:26:49:2e"),
        (
            ECDSA_P256,
            "02:24:b6:70:00:71:04:22:6e:84:c2:87:fa:cc:1c:ff",
        ),
        (
            ECDSA_P384,
            "36:80:0c:ee:e9:06:02:2b:46:7c:cd:53:9a:8f:53:7b",
        ),
        (
            ECDSA_P521,
            "bf:0f:52:31:69:59:ee:9c:53:60:25:b7:8c:20:4d:34",
        ),
        (
            SK_ED25519,
            "82:72:2d:96:a2:2c:45:34:0d:1a:21:f3:9a:e7:69:35",
        ),
        (
            SK_ECDSA_P256,
            "1e:cd:f3:cd:99:68:3f:38:15:84:de:df:bb:c5:c1:75",
        ),
    ] {
        let mut value = resource_value("ssh_key");
        let Some(fields) = value.as_object_mut() else {
            unreachable!("SSH-key fixture is not an object")
        };
        fields.insert("public_key".into(), serde_json::json!(public_key));
        fields.insert("fingerprint".into(), serde_json::json!(fingerprint));
        let body = serde_json::to_vec(&serde_json::json!({"ssh_key":value})).unwrap_or_default();
        let decoded = decode_response(
            prepared("get_ssh_key", SECURITY_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, &body),
        );
        let Ok(decoded) = decoded else {
            unreachable!("admitted SSH algorithm failed checked decoding")
        };
        let HetznerSuccess::SecurityResource(SecurityResource::SshKey(key)) = decoded.success()
        else {
            unreachable!("SSH-key response selected the wrong model")
        };
        assert_eq!(
            key.try_with_public_key(|value| value == public_key),
            Ok(true)
        );
        assert_ne!(key.sha256_fingerprint(), &[0_u8; 32]);
    }
}
