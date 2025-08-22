use std::io::prelude::*;
use std::time::Duration;

use axum::http::header::CONTENT_TYPE;
use axum::response::IntoResponse;
use axum::response::Response;
use base64::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use chrono::serde::ts_seconds_option;
use crypto::EcdsaKey;
use derive_more::AsRef;
use derive_more::FromStr;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use jwt::Jwt;
use jwt::error::JwtError;
use serde::Deserialize;
use serde::Serialize;
use serde::ser::SerializeStruct;
use serde_repr::Deserialize_repr;
use serde_repr::Serialize_repr;
use serde_with::DurationSeconds;
use serde_with::serde_as;

use http_utils::urls::HttpsUri;

static TOKEN_STATUS_LIST_JWT_TYP: &str = "statuslist+jwt";
static TOKEN_STATUS_LIST_JWT_HEADER: &str = "application/statuslist+jwt";

/// A Status List Token embeds a Status List into a token that is cryptographically signed and protects the integrity of
/// the Status List.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-status-list-token>
#[derive(Debug, Clone, FromStr, Serialize, Deserialize)]
pub struct StatusListToken(Jwt<StatusListClaims>);

impl StatusListToken {
    pub async fn try_new(
        iat: DateTime<Utc>,
        exp: Option<DateTime<Utc>>,
        sub: HttpsUri,
        ttl: Option<Duration>,
        status_list: StatusList,
        key: &impl EcdsaKey,
    ) -> Result<Self, JwtError> {
        let claims = StatusListClaims {
            iat,
            exp,
            sub,
            ttl,
            status_list,
        };
        let header = Header {
            typ: Some(TOKEN_STATUS_LIST_JWT_TYP.to_string()),
            ..Header::new(Algorithm::ES256)
        };

        let jwt = Jwt::sign(&claims, &header, key).await?;
        Ok(StatusListToken(jwt))
    }
}

impl IntoResponse for StatusListToken {
    fn into_response(self) -> Response {
        let mut response = self.0.0.into_response();
        response
            .headers_mut()
            .insert(CONTENT_TYPE, TOKEN_STATUS_LIST_JWT_HEADER.parse().unwrap());

        response
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusListClaims {
    #[serde(with = "ts_seconds")]
    iat: DateTime<Utc>,

    #[serde(with = "ts_seconds_option")]
    exp: Option<DateTime<Utc>>,

    /// The sub (subject) claim MUST specify the URI of the Status List Token. The value MUST be equal to that of the
    /// `uri` claim contained in the `status_list` claim of the Referenced Token
    sub: HttpsUri,

    /// If present, MUST specify the maximum amount of time, in seconds, that the Status List Token can be cached by a
    /// consumer before a fresh copy SHOULD be retrieved.
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    ttl: Option<Duration>,

    status_list: StatusList,
}

#[derive(Debug, Clone, PartialEq, Eq, AsRef)]
pub struct StatusList(Vec<StatusType>);

impl Serialize for StatusList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (bits, packed) = self.pack();

        // Implementations are RECOMMENDED to use the highest compression level available.
        let mut e = ZlibEncoder::new(Vec::new(), Compression::best());
        e.write_all(&packed).map_err(serde::ser::Error::custom)?;
        let compressed = e.finish().map_err(serde::ser::Error::custom)?;

        let lst = BASE64_URL_SAFE_NO_PAD.encode(compressed);

        let mut serialized = serializer.serialize_struct("StatusList", 2)?;
        serialized.serialize_field("bits", &bits)?;
        serialized.serialize_field("lst", &lst)?;
        serialized.end()
    }
}

impl<'de> Deserialize<'de> for StatusList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StatusListRaw {
            bits: Bits,
            lst: String,
        }

        let raw = StatusListRaw::deserialize(deserializer)?;
        let compressed = BASE64_URL_SAFE_NO_PAD
            .decode(raw.lst)
            .map_err(serde::de::Error::custom)?;

        let mut d = ZlibDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        d.read_to_end(&mut decompressed).map_err(serde::de::Error::custom)?;

        let status_list = StatusList::unpack(raw.bits, &decompressed);
        Ok(status_list)
    }
}

impl StatusList {
    fn pack(&self) -> (Bits, Vec<u8>) {
        let bits = self
            .0
            .iter()
            .max_by(|a, b| a.bits().cmp(&b.bits()))
            .map(|s| s.bits())
            .unwrap_or_default(); // empty list

        let level = 8 / bits as usize;

        let mut lst = vec![0; (self.0.len() * bits as usize).div_ceil(8)];
        for (index, status) in self.0.iter().enumerate() {
            lst[index / level] |= status.as_u8() << (index % level);
        }

        (bits, lst)
    }

    fn unpack(bits: Bits, lst: &[u8]) -> Self {
        let level = 8 / bits as usize;
        let mask = (2_u16.pow(bits as u32) - 1) as u8;

        let lst = lst
            .iter()
            .flat_map(|byte| {
                (0..level).map(move |i| {
                    let status = byte >> (i * bits as usize) & mask;
                    StatusType::from(status)
                })
            })
            .collect();

        StatusList(lst)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Bits {
    #[default]
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
}

/// A status describes the state, mode, condition or stage of an entity that is represented by the Referenced Token.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-status-types>
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StatusType {
    /// The status of the Referenced Token is valid, correct or legal.
    #[default]
    Valid, // 0x00

    /// The status of the Referenced Token is revoked, annulled, taken back, recalled or cancelled.
    Invalid, // 0x01

    /// The status of the Referenced Token is temporarily invalid, hanging, debarred from privilege. This state is
    /// usually temporary.
    Suspended, // 0x02

    /// The Status Type value 0x03 and Status Type values in the range 0x0B until 0x0F are permanently reserved as
    /// application specific. The processing of Status Types using these values is application specific.
    ApplicationSpecific(u8),

    /// All other Status Type values are reserved for future registration.
    Reserved(u8),
}

impl From<u8> for StatusType {
    fn from(value: u8) -> Self {
        match value {
            0 => StatusType::Valid,
            1 => StatusType::Invalid,
            2 => StatusType::Suspended,
            3 => StatusType::ApplicationSpecific(3),
            0x0B..=0x0F => StatusType::ApplicationSpecific(value),
            _ => StatusType::Reserved(value),
        }
    }
}

impl StatusType {
    fn bits(self) -> Bits {
        match self {
            StatusType::Valid | StatusType::Invalid => Bits::One,
            StatusType::Suspended | StatusType::ApplicationSpecific(3) => Bits::Two,
            StatusType::ApplicationSpecific(i) | StatusType::Reserved(i) if i <= 0x0F => Bits::Four,
            StatusType::Reserved(_) => Bits::Eight,
            _ => panic!("invalid status type"),
        }
    }

    fn as_u8(self) -> u8 {
        match self {
            StatusType::Valid => 0,
            StatusType::Invalid => 1,
            StatusType::Suspended => 2,
            StatusType::ApplicationSpecific(i) | StatusType::Reserved(i) => i,
        }
    }
}

/// By including a "status" claim in a Referenced Token, the Issuer is referencing a mechanism to retrieve status
/// information about this Referenced Token. This specification defines one possible member of the "status" object,
/// called "status_list". Other members of the "status" object may be defined by other specifications.
///
/// ```json
/// "status": {
///     "status_list": {
///         "idx": 0,
///         "uri": "https://example.com/statuslists/1"
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusClaim {
    StatusList(StatusListClaim),
}

/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-referenced-token>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusListClaim {
    /// A non-negative Integer that represents the index to check for status information in the Status List for the
    /// current Referenced Token.
    idx: u32,

    /// URI that identifies the Status List Token containing the status information for the Referenced Token.
    uri: HttpsUri,
}

#[cfg(test)]
mod test {
    use axum::Router;
    use axum::routing::get;
    use regex::Regex;
    use rstest::rstest;
    use serde_json::json;
    use tokio::net::TcpListener;

    use crypto::mock_remote::MockRemoteEcdsaKey;
    use http_utils::urls::BaseUrl;
    use tests_integration::common::wait_for_server;

    use super::*;

    #[test]
    fn test_deserialize_status() {
        let value = json!({
            "status_list": {
                "idx": 0,
                "uri": "https://example.com/statuslists/1"
            }
        });
        let StatusClaim::StatusList(status_claim) = serde_json::from_value(value).unwrap();
        assert_eq!(status_claim.idx, 0);
        assert_eq!(status_claim.uri, "https://example.com/statuslists/1".parse().unwrap());
    }

    const EXAMPLE_STATUS_LIST_ONE: &str = include_str!("../examples/spec/example-status-list-1.txt");
    const EXAMPLE_STATUS_LIST_TWO: &str = include_str!("../examples/spec/example-status-list-2.txt");
    const ONE_BIT_STATUS_LIST: &str = include_str!("../examples/spec/1-bit-status-list.txt");
    const TWO_BIT_STATUS_LIST: &str = include_str!("../examples/spec/2-bit-status-list.txt");
    const FOUR_BIT_STATUS_LIST: &str = include_str!("../examples/spec/4-bit-status-list.txt");
    const EIGHT_BIT_STATUS_LIST: &str = include_str!("../examples/spec/8-bit-status-list.txt");

    fn parse_status_list(input: &str) -> StatusList {
        let re = Regex::new(r"status\[(\d+)\]\s*=\s*(\d+)").unwrap();

        let mut max_index = 0;
        let mut result = Vec::new();
        for cap in re.captures_iter(input) {
            let idx = cap.get(1).unwrap().as_str().parse::<usize>().unwrap();
            let value = cap.get(2).unwrap().as_str().parse::<u8>().unwrap();

            if idx + 1 > max_index {
                result.resize(idx + 1, StatusType::Valid);
                max_index = idx;
            }

            result[idx] = StatusType::from(value);
        }

        StatusList(result)
    }

    #[rstest]
    #[case(parse_status_list(EXAMPLE_STATUS_LIST_ONE), Bits::One)]
    #[case(parse_status_list(EXAMPLE_STATUS_LIST_TWO), Bits::Two)]
    #[case(parse_status_list(ONE_BIT_STATUS_LIST), Bits::One)]
    #[case(parse_status_list(TWO_BIT_STATUS_LIST), Bits::Two)]
    #[case(parse_status_list(FOUR_BIT_STATUS_LIST), Bits::Four)]
    #[case(parse_status_list(EIGHT_BIT_STATUS_LIST), Bits::Eight)]
    fn test_status_list_serialization(#[case] list: StatusList, #[case] expected: Bits) {
        let compressed = serde_json::to_value(list).unwrap();
        assert_eq!(compressed["bits"].as_u64().unwrap(), expected as u64);
    }

    #[rstest]
    #[case(json!({
            "bits": 1,
            "lst": "eNrbuRgAAhcBXQ",
        }),
        parse_status_list(EXAMPLE_STATUS_LIST_ONE)
    )]
    #[case(json!({
            "bits": 2,
            "lst": "eNo76fITAAPfAgc"
        }),
        parse_status_list(EXAMPLE_STATUS_LIST_TWO)
    )]
    #[case(json!({
            "bits": 1,
            "lst":
                "eNrt3AENwCAMAEGogklACtKQPg9LugC9k_ACvreiogEAAKkeCQAAAAAAAAAAAAAAAAAAAIBylgQAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAXG9IAAAAAAAAAPwsJAAAAAAAAAAAAAAAvhsSAAAAAAAAAAAA7KpLAAAAAAAAAAAAAAAAAAAAAJ\
                 sLCQAAAAAAAAAAADjelAAAAAAAAAAAKjDMAQAAAACAZC8L2AEb"
        }),
        parse_status_list(ONE_BIT_STATUS_LIST)
    )]
    #[case(json!({
            "bits": 2,
            "lst":
                "eNrt2zENACEQAEEuoaBABP5VIO01fCjIHTMStt9ovGVIAAAAAABAbiEBAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEB5W\
                 wIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAID0ugQAAAAAAAAAAAAAAAAAQG12SgAAAAAAAAAAAAAAAAAAAAAAAAAAAOCSIQEAAAAAAAAAAAAAAAAAAAAAAAD8ExIAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAwJEuAQAAAAAAAAAAAAAAAAAAAAAAAMB9SwIAAAAAAAAAAAAAAAAAAACoYUoAAAAA\
                 AAAAAAAAAEBqH81gAQw"
        }),
        parse_status_list(TWO_BIT_STATUS_LIST)
    )]
    #[case(json!({
            "bits": 4,
            "lst":
                "eNrt0EENgDAQADAIHwImkIIEJEwCUpCEBBQRHOy35Li1EjoOQGabAgAAAAAAAAAAAAAAAAAAACC1SQEAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABADrsCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADoxaEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIIoCgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAACArpwKAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGhqVkAzlwIAAAAAiGVRAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAAAABx3AoAgLpVAQAAAAAAAAAAAAAAwM89rwMAAAAAAAAAAAjsA9xMBMA"
        }),
        parse_status_list(FOUR_BIT_STATUS_LIST)
    )]
    #[case(json!({
            "bits": 8,
            "lst":
                "eNrt0WOQM2kYhtGsbdu2bdu2bdu2bdu2bdu2jVnU1my-SWYm6U5enFPVf7ue97orFYAo7CQBAACQuuckAABStqUEAAAAAAAAtN6wE\
                 gAE71QJAAAAAIrwhwQAAAAAAdtAAgAAAAAAACLwkAQAAAAAAAAAAACUaFcJAACAeJwkAQAAAAAAAABQvL4kAAAAWmJwCQAAAAAAAA\
                 jAwBIAAAB06ywJoDKQBARpfgkAAAAAAAAAAAAAAAAAAACo50sJAAAAAAAAAOiRcSQAAAAAgAJNKgEAAG23mgQAAAAAAECw3pUAQve\
                 gBAAAAAAAAADduE4CAAAAyjSvBAAQiw8koHjvSABAb-wlARCONyVoxtMSZOd0CQAAAOjWDRKQmLckAAAAAACysLYEQGcnSAAAAAAQ\
                 ooUlAABI15kSAIH5RAIgLB9LABC4_SUgGZNIAABAmM6RoLbTJIASzCIBAEAhfpcAAAAAAABquk8CAAAAAAAAaJl9SvvzBOICAFWmk\
                 IBgfSgBAAAANOgrCQAAAAAAAADStK8EAAC03gASAAAAAAAAAADFWFUCAAAAMjOaBEADHpYAQjCIBADduFwCAAAAAGitMSSI3BUSAE\
                 COHpAA6IHrJQAAAAAAsjeVBAAAKRpVAorWvwQAAAAAAAAAkKRtJAAAAAAAgCbcLAF0bXUJAAAAoF02kYDg7CYBAAAAAEB6NpQAAAA\
                 AAAAAAAAAAEr1uQQAAF06VgIAAAAAAAAAqDaeBAAQqgMkAAAAAABogQMlAAAAAAAa87MEAAAQiwslAAAAAAAAAAAAAAAAMrOyBAAA\
                 iekv-hcsY0Sgne6QAAAAAAAgaUtJAAAAAAAAAAAAAAAAAAAAAAAAAADwt-07vjVkAAAAgDy8KgFAUEaSAAAAAJL3vgQAWdhcAgAAo\
                 BHDSUDo1pQAAACI2o4SAABZm14CALoyuwQAAPznGQkgZwdLAAAQukclAAAAAAAAAAAAgKbMKgEAAAAAAAAAAAAAAAAAAECftpYAAA\
                 AAAAAAAAAACnaXBAAAAADk7iMJAAAAAAAAAABqe00CAnGbBBG4TAIAgFDdKgFAXCaWAAAAAAAAAAAAAAAAAKAJQwR72XbGAQAAAKA\
                 hh0sAAAAAAABQgO8kAAAAAAAAAAAAACAaM0kAAAC5W0QCAIJ3mAQAxGwxCQAA6nhSAsjZBRIAANEbWQIAAAAAaJE3JACAwA0qAUBI\
                 VpKAlphbAiAPp0iQnKEkAAAAAAAgBP1KAAAAdOl4CQAAAAAAAPjLZBIAAG10RtrPm8_CAEBMTpYAAAAAAIjQYBL8z5QSAAAAAEDYP\
                 pUAACAsj0gAAADQkHMlAAjHDxIA0Lg9JQAAgHDsLQEAAABAQS6WAAAAgLjNFs2l_RgLAIAEfCEBlGZZCQAAaIHjJACgtlskAAAozb\
                 0SAAAAVFtfAgAAAAAAAAAAAAAAAAAAAAAAAKDDtxIAAAAAVZaTAKB5W0kAANCAsSUgJ0tL0GqHSNBbL0gAZflRAgCARG0kQXNmlgC\
                 ABiwkAQAAAEB25pIAAAAAAAAAAAAAoFh9SwAAAAAAADWNmOSrpjFsEoaRgDKcF9Q1dxsEAAAAAAAAAAAAAAAAgPZ6SQIAAAAAAAAA\
                 gChMLgEAAAAAAAAAqZlQAsK2qQQAAAAAAAD06XUJAAAAqG9bCQAAgLD9IgEAAAAAAAAAAAAAAAAAAEBNe0gAAAAAAAAAAEBPHSEBA\
                 AAAlOZtCYA4fS8B0GFRCQAo0gISAOTgNwmC840EAAAAAAAAAAAAAAAAAAAAUJydJfjXPBIAAAAAAAAAAAAAAABk6WwJAAAAAAAAAA\
                 AAAAAAqG8UCQAAgPpOlAAAIA83SQAANWwc9HUjGAgAAAAAAACAusaSAAAAAAAAAAAAAAAAAAAAAAAAAAAAqHKVBACQjxklAAAAAAA\
                 AAKBHxpQAAAAAACBME0lAdlaUAACyt7sEAAAA0Nl0EgAAAAAAAAAAAABA-8wgAQAAAAAAAKU4SgKgUtlBAgAAAAAAAAAAgMCMLwEE\
                 51kJICdzSgCJGl2CsE0tAQAA0L11JQAAAAAAAAjUOhIAAAAAAAAAAAAAAGTqeQkAAAAAAAAAAAAAKM8SEjTrJwkAAAAAAACocqQEU\
                 LgVJAAAACjDUxJUKgtKAAAAqbpRAgCA0n0mAQAAAABAGzwmAUCTLpUAAAAAAAAAAEjZNRIAAAAAAAAAAAAAAAAAAAAA8I-vJaAlhp\
                 QAAAAAAHrvzjJ-OqCuuVlLAojP8BJAr70sQZVDJYAgXS0BAAAAAAAAAAAAtMnyEgAAAAAAFONKCQAAAAAAAADorc0kAAAAAAAAgDq\
                 OlgAAAAAAAAAAAADIwv0SAAAAAAAAAAAAAADBuV0CIFVDSwAAAABAAI6RAAAAAGIwrQSEZAsJAABouRclAAAAAKDDrxIAAAA0bkkJ\
                 gFiMKwEAAAAAAHQyhwRk7h4JAAAAAAAAAAAgatdKAACUYj0JAAAAAAAAAAAAQnORBLTFJRIAAAAAkIaDJAAAAJryngQAAAAAAAAAA\
                 AA98oQEAAAAAAAAAEC2zpcgWY9LQKL2kwAgGK9IAAAAAPHaRQIAAAAAAAAAAADIxyoSAAAAAAAAAAAAAADQFotLAECz_gQ1PX-B"
        }),
        parse_status_list(EIGHT_BIT_STATUS_LIST)
    )]
    fn test_status_list_deserialization(#[case] value: serde_json::Value, #[case] expected: StatusList) {
        let status_list: StatusList = serde_json::from_value(value).unwrap();

        assert_eq!(status_list.0[..expected.0.len()], expected.0);
        // everything not in the expected list should be Valid
        assert_eq!(
            status_list.0[expected.0.len()..],
            vec![StatusType::Valid; status_list.0.len() - expected.0.len()]
        );
    }

    #[tokio::test]
    async fn test_status_list_token() {
        let example_header = json!({
            "alg": "ES256",
            "kid": "12",
            "typ": "statuslist+jwt"
        });
        let example_payload = json!({
            "exp": 2291720170_i64,
            "iat": 1686920170,
            "status_list": {
                "bits": 1,
                "lst": "eNrbuRgAAhcBXQ"
            },
            "sub": "https://example.com/statuslists/1",
            "ttl": 43200
        });

        let header: Header = serde_json::from_value(example_header).unwrap();
        assert_eq!(header.typ, Some(TOKEN_STATUS_LIST_JWT_TYP.to_string()));

        let claims: StatusListClaims = serde_json::from_value(example_payload).unwrap();

        let key = MockRemoteEcdsaKey::new_random("key".to_string());
        let signed = StatusListToken::try_new(claims.iat, claims.exp, claims.sub, claims.ttl, claims.status_list, &key)
            .await
            .unwrap();

        let (header, _) = signed
            .0
            .parse_and_verify_with_header(&key.verifying_key().into(), &jwt::validations())
            .unwrap();
        assert_eq!(header.typ, Some(TOKEN_STATUS_LIST_JWT_TYP.to_string()));
    }

    async fn start_mock_server() -> BaseUrl {
        let listener = TcpListener::bind("localhost:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let token_status_list = StatusListToken::try_new(
            Utc::now(),
            Some(Utc::now() + Duration::from_secs(3600)),
            "https://example.com/statuslists/1".parse().unwrap(),
            Some(Duration::from_secs(43200)),
            parse_status_list(EXAMPLE_STATUS_LIST_ONE),
            &MockRemoteEcdsaKey::new_random("key".to_string()),
        )
        .await
        .unwrap();

        let app = Router::new()
            .route("/", get(move || async { token_status_list }))
            .into_make_service();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url: BaseUrl = format!("http://localhost:{port}/").as_str().parse().unwrap();
        wait_for_server(url.clone(), std::iter::empty()).await;
        url
    }

    #[tokio::test]
    async fn test_token_status_list_into_response() {
        let url = start_mock_server().await;

        let response = reqwest::Client::new()
            .get(url.into_inner())
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            TOKEN_STATUS_LIST_JWT_HEADER
        );
        let status_list_token: StatusListToken = response.text().await.unwrap().parse().unwrap();
        let (header, payload) = status_list_token.0.dangerous_parse_unverified().unwrap();
        assert_eq!(header.typ.unwrap(), TOKEN_STATUS_LIST_JWT_TYP);
        assert!(!payload.status_list.as_ref().is_empty());
    }
}
