mod client;
mod types;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, Semaphore, SemaphorePermit};
use uuid::Uuid;

use self::client::QuakeClient;
pub use self::client::QuakeError;
pub use self::types::{
    AccountInfo, AggregateArgs, AggregationResult, FieldsArgs, FieldsResult, ScrollArgs,
    SearchArgs, SearchPage,
};
use self::types::{Dataset, FieldPurpose, SearchQuery, Total, TotalRelation};

const CURSOR_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_CURSORS: usize = 128;

pub struct Quake {
    api: QuakeClient,
    in_flight: Semaphore,
    cursors: Mutex<HashMap<String, CursorState>>,
}

impl Quake {
    pub fn new(token: &str) -> Result<Self, &'static str> {
        Ok(Self::with_client(QuakeClient::new(token)?))
    }

    fn with_client(api: QuakeClient) -> Self {
        Self {
            api,
            in_flight: Semaphore::new(1),
            cursors: Mutex::new(HashMap::new()),
        }
    }

    fn try_request(&self) -> Result<SemaphorePermit<'_>, QuakeError> {
        self.in_flight.try_acquire().map_err(|_| {
            QuakeError::new(
                "busy",
                "Another Quake request is in progress. Wait for it to finish.",
            )
        })
    }

    pub async fn search(&self, mut args: SearchArgs) -> Result<SearchPage, QuakeError> {
        args.search.validate(args.dataset)?;
        if args.start > i64::MAX as u64 {
            return Err(QuakeError::new(
                "invalid_parameters",
                "start exceeds the supported offset range.",
            ));
        }
        let _permit = self.try_request()?;
        let payload = self
            .api
            .post(
                &args.dataset.api_path("search"),
                &OffsetRequest {
                    search: &args.search,
                    start: args.start,
                },
            )
            .await?;
        let pagination = parse_meta(&payload.meta);
        let mut page = SearchPage::new(payload.decode()?, pagination.offset_total());
        let next = args.start + page.count as u64;
        page.has_more = match (page.count, page.total, page.total_relation) {
            (0, _, _) => Some(false),
            (_, Some(total), Some(TotalRelation::Eq)) => Some(next < total),
            (_, Some(total), Some(TotalRelation::Gte)) if next < total => Some(true),
            _ => None,
        };
        page.next_start = page.has_more.filter(|more| *more).map(|_| next);
        Ok(page)
    }

    pub async fn scroll(&self, args: ScrollArgs) -> Result<SearchPage, QuakeError> {
        let _permit = self.try_request()?;
        let mut state = match args {
            ScrollArgs::Start(mut args) => {
                args.search.validate(args.dataset)?;
                CursorState {
                    dataset: args.dataset,
                    search: args.search,
                    upstream_id: None,
                    expires_at: None,
                    last_known_total: None,
                }
            }
            ScrollArgs::Next(args) => self.take_cursor(&args.cursor).await?,
        };
        let payload = self
            .api
            .post(
                &state.dataset.api_path("scroll"),
                &ScrollRequest {
                    search: &state.search,
                    pagination_id: state.upstream_id.as_deref(),
                },
            )
            .await?;
        let metadata = parse_meta(&payload.meta);
        let mut page = SearchPage::new(payload.decode()?, metadata.total);
        if page.count == 0 {
            page.has_more = Some(false);
            if page.total.unwrap_or(0) == 0
                && let Some(previous) = state.last_known_total
            {
                let (total, relation) = previous.parts();
                page.total = Some(total);
                page.total_relation = Some(relation);
            }
        } else if let Some(upstream_id) = metadata.pagination_id.filter(|id| !id.is_empty()) {
            state.upstream_id = Some(upstream_id);
            if let Some(total) = metadata.total.filter(|total| total.parts().0 > 0) {
                state.last_known_total = Some(total);
            }
            let deadline = *state
                .expires_at
                .get_or_insert_with(|| Instant::now() + CURSOR_TTL);
            if deadline > Instant::now() {
                page.next_cursor = Some(self.save_cursor(state).await);
                page.has_more = Some(true);
            } else {
                page.warnings
                    .push("The continuation expired while Quake was returning this page.");
            }
        } else {
            page.warnings
                .push("Quake returned records without a continuation cursor.");
        }
        Ok(page)
    }

    async fn take_cursor(&self, handle: &str) -> Result<CursorState, QuakeError> {
        let now = Instant::now();
        self.cursors.lock().await.remove(handle)
            .filter(|entry| entry.expires_at.is_some_and(|expires_at| expires_at > now))
            .ok_or_else(|| QuakeError::new("invalid_cursor", "The cursor is expired, already used, or from another server process. Start a new query."))
    }

    async fn save_cursor(&self, state: CursorState) -> String {
        let mut cursors = self.cursors.lock().await;
        let now = Instant::now();
        cursors.retain(|_, entry| entry.expires_at.is_some_and(|expires_at| expires_at > now));
        if cursors.len() >= MAX_CURSORS {
            let oldest = cursors
                .iter()
                .min_by_key(|(_, value)| value.expires_at)
                .map(|(handle, _)| handle.clone());
            if let Some(handle) = oldest {
                cursors.remove(&handle);
            }
        }
        let handle = Uuid::new_v4().to_string();
        cursors.insert(handle.clone(), state);
        handle
    }

    pub async fn aggregate(&self, args: AggregateArgs) -> Result<AggregationResult, QuakeError> {
        args.aggregation.validate(args.dataset)?;
        let _permit = self.try_request()?;
        let payload = self
            .api
            .post(&args.dataset.api_path("aggregation"), &args.aggregation)
            .await?;
        Ok(AggregationResult {
            buckets: payload.decode()?,
        })
    }

    pub async fn fields(&self, args: FieldsArgs) -> Result<FieldsResult, QuakeError> {
        let _permit = self.try_request()?;
        let operation = match args.purpose {
            FieldPurpose::Search => "filterable/field",
            FieldPurpose::Aggregation => "aggregation",
        };
        let payload = self.api.get(&args.dataset.api_path(operation)).await?;
        Ok(FieldsResult {
            fields: payload.decode()?,
        })
    }

    pub async fn account_info(&self) -> Result<AccountInfo, QuakeError> {
        let _permit = self.try_request()?;
        self.api.get("/api/v3/user/info").await?.decode()
    }
}

#[derive(Serialize)]
struct OffsetRequest<'a> {
    #[serde(flatten)]
    search: &'a SearchQuery,
    start: u64,
}

#[derive(Serialize)]
struct ScrollRequest<'a> {
    #[serde(flatten)]
    search: &'a SearchQuery,
    #[serde(skip_serializing_if = "Option::is_none")]
    pagination_id: Option<&'a str>,
}

struct CursorState {
    dataset: Dataset,
    search: SearchQuery,
    upstream_id: Option<String>,
    expires_at: Option<Instant>,
    last_known_total: Option<Total>,
}

#[derive(Default, Deserialize)]
struct PageMetadata {
    pagination: Option<OffsetMeta>,
    pagination_id: Option<String>,
    total: Option<Total>,
}

#[derive(Deserialize)]
struct OffsetMeta {
    total: Option<Total>,
}

impl PageMetadata {
    fn offset_total(&self) -> Option<Total> {
        self.pagination
            .as_ref()
            .and_then(|pagination| pagination.total)
            .or(self.total)
    }
}

fn parse_meta(meta: &serde_json::Value) -> PageMetadata {
    PageMetadata::deserialize(meta).unwrap_or_default()
}

impl SearchPage {
    fn new(items: Vec<serde_json::Map<String, serde_json::Value>>, total: Option<Total>) -> Self {
        let parts = total.map(Total::parts);
        Self {
            count: items.len(),
            items,
            total: parts.map(|(value, _)| value),
            total_relation: parts.map(|(_, relation)| relation),
            has_more: None,
            next_start: None,
            next_cursor: None,
            warnings: vec![],
        }
    }
}

#[cfg(test)]
mod tests;
