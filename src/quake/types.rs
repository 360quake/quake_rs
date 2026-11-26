use std::collections::BTreeMap;

use chrono::NaiveDateTime;
use rmcp::{schemars, schemars::JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::QuakeError;

pub const MAX_PAGE_SIZE: u32 = 50;
pub const MAX_AGGREGATION_SIZE: u32 = 100;

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Dataset {
    Service,
    Host,
}

impl Dataset {
    pub fn api_path(self, operation: &str) -> String {
        let suffix = match self {
            Self::Service => "quake_service",
            Self::Host => "quake_host",
        };
        format!("/api/v3/{operation}/{suffix}")
    }

    fn validate_latest(self, latest: Option<bool>) -> Result<(), QuakeError> {
        if matches!(self, Self::Host) && latest.is_some() {
            return Err(QuakeError::new(
                "invalid_parameters",
                "latest applies only to the service dataset. Omit it for host data.",
            ));
        }
        Ok(())
    }

    fn default_fields(self) -> Vec<String> {
        let fields: &[&str] = match self {
            Self::Service => &["ip", "port", "hostname", "transport", "service.name"],
            Self::Host => &["ip", "asn", "org", "location.country_cn"],
        };
        fields.iter().map(|field| (*field).to_owned()).collect()
    }
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct SearchQuery {
    /// Quake DSL scoped to the assets being inventoried.
    pub query: String,
    /// Maximum records on this page.
    #[serde(default = "default_page_size")]
    #[schemars(range(min = 1, max = 50))]
    pub size: u32,
    /// Optional UTC timestamp, YYYY-MM-DD HH:MM:SS. Requires an eligible Quake account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// Optional UTC timestamp, YYYY-MM-DD HH:MM:SS. Requires an eligible Quake account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Service dataset only. Set false for historical records, true for latest records, or omit for the Quake default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<bool>,
    /// Quake field names to return. Uses a compact field set if both field lists are omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// Quake field names to omit. Use quake_fields to find valid names for the dataset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

const fn default_page_size() -> u32 {
    10
}

impl SearchQuery {
    pub fn validate(&mut self, dataset: Dataset) -> Result<(), QuakeError> {
        dataset.validate_latest(self.latest)?;
        validate_query_and_times(
            &self.query,
            self.start_time.as_deref(),
            self.end_time.as_deref(),
        )?;
        if !(1..=MAX_PAGE_SIZE).contains(&self.size) {
            return Err(QuakeError::new(
                "invalid_parameters",
                "size must be between 1 and 50 for search.",
            ));
        }
        validate_fields(self.include.as_deref().unwrap_or_default())?;
        validate_fields(self.exclude.as_deref().unwrap_or_default())?;
        if self.include.is_none() && self.exclude.is_none() {
            self.include = Some(dataset.default_fields());
        }
        Ok(())
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SearchArgs {
    pub dataset: Dataset,
    /// Zero-based result offset. Use next_start from the preceding page.
    #[serde(default)]
    pub start: u64,
    #[serde(flatten)]
    pub search: SearchQuery,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScrollStart {
    pub dataset: Dataset,
    #[serde(flatten)]
    pub search: SearchQuery,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScrollNext {
    /// Continuation handle returned by quake_scroll, valid within this server process.
    pub cursor: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("type" = "object"))]
pub enum ScrollArgs {
    Next(ScrollNext),
    Start(ScrollStart),
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AggregateArgs {
    pub dataset: Dataset,
    #[serde(flatten)]
    pub aggregation: AggregationQuery,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct AggregationQuery {
    /// Quake DSL scoped to the assets being inventoried.
    pub query: String,
    /// One or two distinct dimensions from quake_fields with purpose aggregation.
    #[schemars(length(min = 1, max = 2))]
    pub aggregation_list: Vec<String>,
    /// Maximum buckets per dimension.
    #[serde(default = "default_aggregation_size")]
    #[schemars(range(min = 1, max = 100))]
    pub size: u32,
    /// Optional UTC timestamp, YYYY-MM-DD HH:MM:SS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// Optional UTC timestamp, YYYY-MM-DD HH:MM:SS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Service dataset only. Set false for historical records, true for latest records, or omit for the Quake default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest: Option<bool>,
}

const fn default_aggregation_size() -> u32 {
    5
}

impl AggregationQuery {
    pub fn validate(&self, dataset: Dataset) -> Result<(), QuakeError> {
        dataset.validate_latest(self.latest)?;
        validate_query_and_times(
            &self.query,
            self.start_time.as_deref(),
            self.end_time.as_deref(),
        )?;
        if !(1..=MAX_AGGREGATION_SIZE).contains(&self.size) {
            return Err(QuakeError::new(
                "invalid_parameters",
                "size must be between 1 and 100 for aggregation.",
            ));
        }
        match self.aggregation_list.as_slice() {
            [_] => validate_fields(&self.aggregation_list),
            [first, second] if first.trim() != second.trim() => {
                validate_fields(&self.aggregation_list)
            }
            _ => Err(QuakeError::new(
                "invalid_parameters",
                "Provide one or two distinct aggregation fields.",
            )),
        }
    }
}

fn validate_fields(fields: &[String]) -> Result<(), QuakeError> {
    if fields.iter().any(|field| field.trim().is_empty()) {
        return Err(QuakeError::new(
            "invalid_parameters",
            "Field names must not be blank.",
        ));
    }
    Ok(())
}

fn validate_query_and_times(
    query: &str,
    start: Option<&str>,
    end: Option<&str>,
) -> Result<(), QuakeError> {
    if query.trim().is_empty() {
        return Err(QuakeError::new(
            "invalid_query",
            "Provide a non-empty Quake DSL query.",
        ));
    }
    let parse = |time: Option<&str>| {
        time.map(|value| {
            let time = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").ok();
            time.filter(|time| time.format("%Y-%m-%d %H:%M:%S").to_string() == value)
                .ok_or_else(|| {
                    QuakeError::new(
                        "invalid_parameters",
                        "Use UTC timestamps in YYYY-MM-DD HH:MM:SS format.",
                    )
                })
        })
        .transpose()
    };
    if let (Some(start), Some(end)) = (parse(start)?, parse(end)?)
        && start > end
    {
        return Err(QuakeError::new(
            "invalid_parameters",
            "start_time must not be later than end_time.",
        ));
    }
    Ok(())
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldPurpose {
    Search,
    Aggregation,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FieldsArgs {
    pub dataset: Dataset,
    pub purpose: FieldPurpose,
}

#[derive(Serialize, JsonSchema)]
pub struct SearchPage {
    pub items: Vec<Map<String, Value>>,
    pub count: usize,
    pub total: Option<u64>,
    pub total_relation: Option<TotalRelation>,
    pub has_more: Option<bool>,
    pub next_start: Option<u64>,
    pub next_cursor: Option<String>,
    pub warnings: Vec<&'static str>,
}

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TotalRelation {
    Eq,
    Gte,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum Total {
    Count(u64),
    WithRelation {
        value: u64,
        relation: Option<TotalRelation>,
    },
}

impl Total {
    pub fn parts(self) -> (u64, TotalRelation) {
        match self {
            Self::Count(value) => (value, TotalRelation::Eq),
            Self::WithRelation { value, relation } => {
                (value, relation.unwrap_or(TotalRelation::Unknown))
            }
        }
    }
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct Bucket {
    pub key: Value,
    pub doc_count: u64,
    #[serde(flatten)]
    pub details: BTreeMap<String, Value>,
}

#[derive(Serialize, JsonSchema)]
pub struct AggregationResult {
    pub buckets: BTreeMap<String, Vec<Bucket>>,
}

#[derive(Serialize, JsonSchema)]
pub struct FieldsResult {
    pub fields: Vec<String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct AccountInfo {
    #[serde(rename(deserialize = "credit"))]
    pub monthly_credits: Option<u64>,
    #[serde(rename(deserialize = "persistent_credit"))]
    pub persistent_credits: Option<u64>,
    #[serde(rename(deserialize = "baned"))]
    pub blocked: Option<bool>,
}
