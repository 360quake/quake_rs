use std::sync::Arc;

use rmcp::handler::server::{
    common::schema_for_output, router::tool::ToolRouter, wrapper::Parameters,
};
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, schemars, schemars::JsonSchema, tool, tool_handler, tool_router};
use serde::Serialize;

use crate::quake::{
    AccountInfo, AggregateArgs, AggregationResult, FieldsArgs, FieldsResult, Quake, QuakeError,
    ScrollArgs, SearchArgs, SearchPage,
};

#[derive(Clone)]
pub struct QuakeMcp {
    quake: Arc<Quake>,
    router: ToolRouter<Self>,
}

#[tool_handler(router = self.router)]
impl ServerHandler for QuakeMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("quake-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions("Look up existing 360 Quake data for asset inventory. Use a scoped Quake DSL query, a small page, and only the fields needed. Searches and aggregations may consume Quake credits.")
    }
}

#[tool_router(router = router)]
impl QuakeMcp {
    pub fn new(quake: Quake) -> Self {
        Self {
            quake: Arc::new(quake),
            router: Self::router(),
        }
    }

    #[tool(
        description = "Read one offset page of Quake service or host data. Scope the DSL to your inventory, use quake_fields for projections, and pass next_start as start to continue with the same arguments. For historical service records set latest=false and specify the time window. A page can consume credits.",
        output_schema = schema_for_output::<ToolReply<SearchPage>>(),
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn quake_search(&self, Parameters(args): Parameters<SearchArgs>) -> CallToolResult {
        reply(self.quake.search(args).await, page_summary)
    }

    #[tool(
        description = "Read one cursor page of Quake data. Start with dataset and a scoped DSL query. For historical service records set latest=false on the first call. Pass only next_cursor as cursor to continue. Handles are single-use and expire within five minutes. Each page can consume credits.",
        output_schema = schema_for_output::<ToolReply<SearchPage>>(),
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn quake_scroll(&self, Parameters(args): Parameters<ScrollArgs>) -> CallToolResult {
        reply(self.quake.scroll(args).await, page_summary)
    }

    #[tool(
        description = "Get Quake aggregation buckets for a scoped query and one or two dimensions from quake_fields with purpose aggregation. Set latest=false for historical service statistics. May consume credits.",
        output_schema = schema_for_output::<ToolReply<AggregationResult>>(),
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn quake_aggregate(&self, Parameters(args): Parameters<AggregateArgs>) -> CallToolResult {
        reply(self.quake.aggregate(args).await, |data| {
            format!("Returned {} aggregation dimensions.", data.buckets.len())
        })
    }

    #[tool(
        description = "List the live Quake projection fields or aggregation dimensions for the chosen service or host dataset.",
        output_schema = schema_for_output::<ToolReply<FieldsResult>>(),
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn quake_fields(&self, Parameters(args): Parameters<FieldsArgs>) -> CallToolResult {
        reply(self.quake.fields(args).await, |data| {
            format!("Returned {} field names.", data.fields.len())
        })
    }

    #[tool(
        description = "Read the configured Quake account's monthly credits, persistent credits and blocked status.",
        output_schema = schema_for_output::<ToolReply<AccountInfo>>(),
        annotations(read_only_hint = true, open_world_hint = true)
    )]
    async fn quake_account_info(&self) -> CallToolResult {
        reply(self.quake.account_info().await, |_| {
            "Returned Quake account credit and status information.".to_owned()
        })
    }
}

#[derive(Serialize, JsonSchema)]
struct ToolReply<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<QuakeError>,
}

fn reply<T: Serialize>(
    result: Result<T, QuakeError>,
    summarize: impl FnOnce(&T) -> String,
) -> CallToolResult {
    let (body, summary) = match result {
        Ok(data) => {
            let summary = summarize(&data);
            (
                ToolReply {
                    data: Some(data),
                    error: None,
                },
                summary,
            )
        }
        Err(error) => {
            let summary = error.message.to_owned();
            (
                ToolReply {
                    data: None,
                    error: Some(error),
                },
                summary,
            )
        }
    };
    let is_error = body.error.is_some();
    let Ok(value) = serde_json::to_value(body) else {
        return CallToolResult::error(vec![ContentBlock::text(
            "Could not encode the tool result.",
        )]);
    };
    let mut reply = if is_error {
        CallToolResult::structured_error(value)
    } else {
        CallToolResult::structured(value)
    };
    reply.content.insert(0, ContentBlock::text(summary));
    reply
}

fn page_summary(page: &SearchPage) -> String {
    match page.has_more {
        Some(true) => format!(
            "Returned {} records. A continuation is available.",
            page.count
        ),
        Some(false) => format!("Returned {} records. This is the last page.", page.count),
        None => format!(
            "Returned {} records. Further results are unknown.",
            page.count
        ),
    }
}
