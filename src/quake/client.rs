use std::time::{Duration, SystemTime};

use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest::{Client, RequestBuilder, StatusCode};
use rmcp::{schemars, schemars::JsonSchema};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const API_BASE_URL: &str = "https://quake.360.net";
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

pub struct QuakeClient {
    http: Client,
    base_url: String,
}

impl QuakeClient {
    pub fn new(token: &str) -> Result<Self, &'static str> {
        Self::with_url(token, API_BASE_URL, Duration::from_secs(30))
    }

    pub(super) fn with_url(
        token: &str,
        base_url: &str,
        timeout: Duration,
    ) -> Result<Self, &'static str> {
        let token = token.trim();
        if token.is_empty() {
            return Err("QUAKE_TOKEN is empty");
        }
        let mut token = HeaderValue::from_str(token)
            .map_err(|_| "QUAKE_TOKEN is not a valid HTTP header value")?;
        token.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert("x-quaketoken", token);
        let http = Client::builder()
            .default_headers(headers)
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .build()
            .map_err(|_| "could not initialize the Quake HTTP client")?;
        Ok(Self {
            http,
            base_url: base_url.to_owned(),
        })
    }

    pub(super) async fn get(&self, path: &str) -> Result<Payload, QuakeError> {
        self.send(self.http.get(format!("{}{path}", self.base_url)))
            .await
    }

    pub(super) async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<Payload, QuakeError> {
        self.send(
            self.http
                .post(format!("{}{path}", self.base_url))
                .json(body),
        )
        .await
    }

    async fn send(&self, request: RequestBuilder) -> Result<Payload, QuakeError> {
        let mut response = request.send().await.map_err(QuakeError::transport)?;
        let status = response.status();
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(retry_after_seconds);
        if !status.is_success() {
            return Err(QuakeError::http(status, retry_after));
        }

        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(QuakeError::transport)? {
            if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
                return Err(QuakeError::new(
                    "response_too_large",
                    "Quake returned more than 4 MiB. Try a smaller page or fewer fields.",
                ));
            }
            body.extend_from_slice(&chunk);
        }

        let envelope: Envelope =
            serde_json::from_slice(&body).map_err(|_| QuakeError::invalid_response())?;
        if !envelope.code.is_success() {
            return Err(QuakeError::business(envelope.code, retry_after));
        }
        let data = envelope.data.ok_or_else(QuakeError::invalid_response)?;
        Ok(Payload {
            data,
            meta: envelope.meta.unwrap_or_default(),
        })
    }
}

fn retry_after_seconds(value: &HeaderValue) -> Option<u64> {
    let value = value.to_str().ok()?;
    value.parse().ok().or_else(|| {
        let date = httpdate::parse_http_date(value).ok()?;
        Some(
            date.duration_since(SystemTime::now())
                .unwrap_or_default()
                .as_secs(),
        )
    })
}

#[derive(Deserialize)]
struct Envelope {
    code: UpstreamCode,
    data: Option<Value>,
    meta: Option<Value>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum UpstreamCode {
    Number(i64),
    Text(String),
}

impl UpstreamCode {
    fn is_success(&self) -> bool {
        match self {
            Self::Number(number) => *number == 0,
            Self::Text(text) => text == "0",
        }
    }

    fn safe_code(self) -> Option<String> {
        match self {
            Self::Number(number) => Some(number.to_string()),
            Self::Text(text) => {
                let code = text.to_ascii_lowercase();
                let digits = code.strip_prefix(['q', 'u']).unwrap_or(&code);
                (!digits.is_empty()
                    && digits.len() <= 10
                    && digits.bytes().all(|byte| byte.is_ascii_digit()))
                .then_some(code)
            }
        }
    }
}

pub(super) struct Payload {
    data: Value,
    pub(super) meta: Value,
}

impl Payload {
    pub(super) fn decode<T: DeserializeOwned>(self) -> Result<T, QuakeError> {
        serde_json::from_value(self.data).map_err(|_| QuakeError::invalid_response())
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct QuakeError {
    pub code: &'static str,
    pub message: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    upstream_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    http_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after_seconds: Option<u64>,
}

impl QuakeError {
    pub(super) fn new(code: &'static str, message: &'static str) -> Self {
        Self {
            code,
            message,
            upstream_code: None,
            http_status: None,
            retry_after_seconds: None,
        }
    }

    fn http(status: StatusCode, retry_after: Option<u64>) -> Self {
        let mut error = match status {
            StatusCode::UNAUTHORIZED => {
                Self::new("authentication", "Quake rejected the API token.")
            }
            StatusCode::FORBIDDEN => Self::new(
                "permission_denied",
                "The account cannot access this Quake operation.",
            ),
            StatusCode::TOO_MANY_REQUESTS => Self::rate_limited(),
            StatusCode::REQUEST_TIMEOUT | StatusCode::GATEWAY_TIMEOUT => Self::timeout(),
            _ => Self::new(
                "upstream_http",
                "Quake returned an unsuccessful HTTP status.",
            ),
        };
        error.http_status = Some(status.as_u16());
        error.retry_after_seconds = retry_after;
        error
    }

    fn business(code: UpstreamCode, retry_after: Option<u64>) -> Self {
        let code = code.safe_code();
        let mut error = match code.as_deref() {
            Some("3004" | "q3004") => Self::new(
                "verification_required",
                "Quake requires account verification.",
            ),
            Some("3005" | "q3005") => Self::rate_limited(),
            Some("3007" | "q3007") => Self::new(
                "quota_exhausted",
                "The Quake account has insufficient credits.",
            ),
            Some("3009" | "u3009" | "3010" | "u3010") => Self::new(
                "invalid_parameters",
                "Quake rejected the request parameters.",
            ),
            Some("3011" | "q3011") => Self::new(
                "permission_denied",
                "The account lacks the required Quake permission.",
            ),
            Some("3015" | "q3015") => {
                Self::new("invalid_query", "Quake could not parse the DSL query.")
            }
            Some("3017" | "q3017") => Self::new(
                "unsupported_field",
                "Quake rejected a field. Check the field list for this dataset and operation.",
            ),
            _ => Self::new(
                "upstream_error",
                "Quake returned an unsuccessful business code.",
            ),
        };
        error.upstream_code = code;
        error.retry_after_seconds = retry_after;
        error
    }

    fn rate_limited() -> Self {
        Self::new(
            "rate_limited",
            "Quake rate-limited the request. Check Retry-After or the account status before trying again.",
        )
    }

    fn timeout() -> Self {
        Self::new(
            "timeout",
            "Quake timed out. The request may have been processed or charged.",
        )
    }

    fn transport(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            return Self::timeout();
        }
        Self::new(
            "transport_error",
            "The Quake request did not complete. Its processing status may be unknown.",
        )
    }

    fn invalid_response() -> Self {
        Self::new(
            "invalid_response",
            "Quake returned an unexpected response format.",
        )
    }
}
