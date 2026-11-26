use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::*;

const TOKEN: &str = "test-token";
const QUERY: &str = "domain:\"example.com\"";

fn adapter(server: &MockServer, timeout: Duration) -> Quake {
    Quake::with_client(QuakeClient::with_url(TOKEN, &server.uri(), timeout).unwrap())
}

fn input<T: DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn ok(data: Value, meta: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({"code": 0, "data": data, "meta": meta}))
}

#[tokio::test]
async fn offset_search_uses_service_defaults_and_host_overrides() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/search/quake_service"))
        .and(header("X-QuakeToken", TOKEN))
        .and(body_json(json!({"query": QUERY, "start": 0, "size": 10,
            "include": ["ip", "port", "hostname", "transport", "service.name"]})))
        .respond_with(ok(
            json!([{"ip": "192.0.2.1", "port": 443}]),
            json!({"pagination": {"total": 24}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v3/search/quake_host"))
        .and(header("X-QuakeToken", TOKEN))
        .and(body_json(json!({"query": QUERY, "start": 21, "size": 4, "include": ["ip"],
            "exclude": ["org"], "start_time": "2026-01-01 00:00:00", "end_time": "2026-02-01 00:00:00"})))
        .respond_with(ok(json!([{"ip": "192.0.2.2"}]), Value::Null))
        .expect(1).mount(&server).await;

    let quake = adapter(&server, Duration::from_secs(5));
    let service = quake
        .search(input(json!({"dataset": "service", "query": QUERY})))
        .await
        .unwrap();
    assert_eq!(service.count, 1);
    assert_eq!(service.total, Some(24));
    assert_eq!(service.next_start, Some(1));
    let host = quake
        .search(input(
            json!({"dataset": "host", "query": QUERY, "start": 21,
        "size": 4, "include": ["ip"], "exclude": ["org"],
        "start_time": "2026-01-01 00:00:00", "end_time": "2026-02-01 00:00:00"}),
        ))
        .await
        .unwrap();
    assert_eq!(host.items[0]["ip"], "192.0.2.2");
    assert_eq!(host.total, None);
    assert_eq!(host.has_more, None);
}

#[tokio::test]
async fn service_scroll_preserves_the_request_and_accepts_an_unchanged_upstream_id() {
    let server = MockServer::start().await;
    let path_name = "/api/v3/scroll/quake_service";
    let upstream_id = "service-upstream-id";
    let original = json!({"query": QUERY, "size": 1, "include": ["ip"],
        "start_time": "2026-01-01 00:00:00", "latest": false});
    let mut continuation = original.clone();
    continuation["pagination_id"] = json!(upstream_id);
    let metadata = json!({"total": {"value": 8_078_638_343_u64, "relation": "gte"}, "pagination_id": upstream_id});
    Mock::given(method("POST"))
        .and(path(path_name))
        .and(body_json(original))
        .respond_with(ok(json!([{"ip": "192.0.2.4"}]), metadata.clone()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(path_name))
        .and(body_json(continuation))
        .respond_with(ok(json!([{"ip": "192.0.2.5"}]), metadata))
        .expect(1)
        .mount(&server)
        .await;

    let quake = adapter(&server, Duration::from_secs(5));
    let first = quake
        .scroll(input(
            json!({"dataset": "service", "query": QUERY, "size": 1,
        "include": ["ip"], "start_time": "2026-01-01 00:00:00", "latest": false}),
        ))
        .await
        .unwrap();
    assert_eq!(first.total, Some(8_078_638_343));
    assert!(matches!(first.total_relation, Some(TotalRelation::Gte)));
    let handle = first.next_cursor.unwrap();
    assert_ne!(handle, upstream_id);
    let second = quake
        .scroll(input(json!({"cursor": handle})))
        .await
        .unwrap();
    assert_eq!(second.items[0]["ip"], "192.0.2.5");
    assert_ne!(second.next_cursor.as_deref(), Some(handle.as_str()));
    assert!(second.next_cursor.is_some());
    let error = quake
        .scroll(input(json!({"cursor": handle})))
        .await
        .err()
        .unwrap();
    assert_eq!(error.code, "invalid_cursor");
}

#[tokio::test]
async fn host_scroll_handles_empty_page_and_expired_handle() {
    let server = MockServer::start().await;
    let path_name = "/api/v3/scroll/quake_host";
    let start = json!({"query": QUERY, "size": 1, "include": ["ip"]});
    let mut continuation = start.clone();
    continuation["pagination_id"] = json!("host-upstream-id");
    Mock::given(method("POST"))
        .and(path(path_name))
        .and(body_json(start))
        .respond_with(ok(
            json!([{"ip": "192.0.2.8"}]),
            json!({"total": 1, "pagination_id": "host-upstream-id"}),
        ))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(path_name))
        .and(body_json(continuation))
        .respond_with(ok(
            json!([]),
            json!({"total": 0, "pagination_id": "host-upstream-id"}),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let quake = adapter(&server, Duration::from_secs(5));
    let args = json!({"dataset": "host", "query": QUERY, "size": 1, "include": ["ip"]});
    let first = quake.scroll(input(args.clone())).await.unwrap();
    assert_eq!(first.total, Some(1));
    let last = quake
        .scroll(input(json!({"cursor": first.next_cursor.unwrap()})))
        .await
        .unwrap();
    assert!(last.items.is_empty());
    assert_eq!(last.total, Some(1));
    assert!(matches!(last.total_relation, Some(TotalRelation::Eq)));
    assert_eq!(last.has_more, Some(false));
    assert_eq!(last.next_cursor, None);
    let other = quake
        .scroll(input(args))
        .await
        .unwrap()
        .next_cursor
        .unwrap();
    quake
        .cursors
        .lock()
        .await
        .get_mut(&other)
        .unwrap()
        .expires_at = Some(Instant::now());
    assert_eq!(
        quake
            .scroll(input(json!({"cursor": other})))
            .await
            .err()
            .unwrap()
            .code,
        "invalid_cursor"
    );
}

#[tokio::test]
async fn service_latest_preserves_all_three_request_modes() {
    let server = MockServer::start().await;
    let quake = adapter(&server, Duration::from_secs(5));
    for latest in [None, Some(false), Some(true)] {
        let mut search = json!({"query": QUERY, "size": 1, "start": 0, "include": ["ip"]});
        let mut aggregate = json!({"query": QUERY, "size": 1, "aggregation_list": ["port"]});
        if let Some(latest) = latest {
            search["latest"] = json!(latest);
            aggregate["latest"] = json!(latest);
        }
        Mock::given(method("POST"))
            .and(path("/api/v3/search/quake_service"))
            .and(body_json(search.clone()))
            .respond_with(ok(json!([]), json!({"pagination": {"total": 0}})))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/api/v3/aggregation/quake_service"))
            .and(body_json(aggregate.clone()))
            .respond_with(ok(json!({"port": []}), Value::Null))
            .expect(1)
            .mount(&server)
            .await;

        search["dataset"] = json!("service");
        aggregate["dataset"] = json!("service");
        let page = quake.search(input(search)).await.unwrap();
        assert_eq!(page.total, Some(0));
        assert_eq!(page.has_more, Some(false));
        assert!(quake.aggregate(input(aggregate)).await.unwrap().buckets["port"].is_empty());
    }
}

#[tokio::test]
async fn returns_field_lists_buckets_and_only_safe_account_fields() {
    let server = MockServer::start().await;
    let quake = adapter(&server, Duration::from_secs(5));
    for (dataset, suffix) in [("host", "quake_host"), ("service", "quake_service")] {
        let aggregate_path = format!("/api/v3/aggregation/{suffix}");
        Mock::given(method("GET"))
            .and(path(format!("/api/v3/filterable/field/{suffix}")))
            .respond_with(ok(json!(["ip"]), Value::Null))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(&aggregate_path))
            .respond_with(ok(json!(["org"]), Value::Null))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(&aggregate_path))
            .and(body_json(
                json!({"query": QUERY, "aggregation_list": ["org"], "size": 5}),
            ))
            .respond_with(ok(
                json!({"org": [{"key": "example", "doc_count": 3}]}),
                Value::Null,
            ))
            .expect(1)
            .mount(&server)
            .await;
        let fields = quake
            .fields(input(json!({"dataset": dataset, "purpose": "search"})))
            .await
            .unwrap();
        assert_eq!(fields.fields, ["ip"]);
        let dimensions = quake
            .fields(input(json!({"dataset": dataset, "purpose": "aggregation"})))
            .await
            .unwrap();
        assert_eq!(dimensions.fields, ["org"]);
        let result = quake
            .aggregate(input(
                json!({"dataset": dataset, "query": QUERY, "aggregation_list": ["org"]}),
            ))
            .await
            .unwrap();
        assert_eq!(result.buckets["org"][0].doc_count, 3);
    }
    Mock::given(method("GET")).and(path("/api/v3/user/info"))
        .respond_with(ok(json!({"credit": 12, "persistent_credit": 10, "baned": false,
            "token": TOKEN, "user": {"email": "private@example.com"}, "mobile_phone": "private-number"}), Value::Null))
        .expect(1).mount(&server).await;
    let account = serde_json::to_value(quake.account_info().await.unwrap()).unwrap();
    assert_eq!(
        account,
        json!({"monthly_credits": 12, "persistent_credits": 10, "blocked": false})
    );
}

#[tokio::test]
async fn classifies_business_http_and_timeout_failures_without_retrying() {
    let server = MockServer::start().await;
    let quake = adapter(&server, Duration::from_millis(250));
    Mock::given(method("GET"))
        .and(path("/api/v3/user/info"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"code": "q3007", "message": TOKEN})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v3/filterable/field/quake_service"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "17"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v3/search/quake_service"))
        .respond_with(ok(json!([]), Value::Null).set_delay(Duration::from_millis(800)))
        .expect(1)
        .mount(&server)
        .await;

    let quota = serde_json::to_value(quake.account_info().await.err().unwrap()).unwrap();
    assert_eq!(quota["code"], "quota_exhausted");
    assert_eq!(quota["upstream_code"], "q3007");
    assert!(!quota.to_string().contains(TOKEN));
    let rate = quake
        .fields(input(json!({"dataset": "service", "purpose": "search"})))
        .await
        .err()
        .unwrap();
    let rate = serde_json::to_value(rate).unwrap();
    assert_eq!(rate["code"], "rate_limited");
    assert_eq!(rate["retry_after_seconds"], 17);
    let timeout = quake
        .search(input(json!({"dataset": "service", "query": QUERY})))
        .await
        .err()
        .unwrap();
    assert_eq!(timeout.code, "timeout");
    server.verify().await;
}

#[tokio::test]
async fn rejects_invalid_arguments_before_sending_quake_requests() {
    let server = MockServer::start().await;
    let quake = adapter(&server, Duration::from_secs(5));
    let bad_range = quake
        .search(input(json!({"dataset": "host", "query": QUERY,
        "start_time": "2026-02-01 00:00:00", "end_time": "2026-01-01 00:00:00"})))
        .await
        .err()
        .unwrap();
    assert_eq!(bad_range.code, "invalid_parameters");
    let bad_size = quake
        .search(input(
            json!({"dataset": "service", "query": QUERY, "size": 51}),
        ))
        .await
        .err()
        .unwrap();
    assert_eq!(bad_size.code, "invalid_parameters");
    let bad_fields = quake
        .aggregate(input(json!({"dataset": "service", "query": QUERY,
        "aggregation_list": ["org", "port", "service"]})))
        .await
        .err()
        .unwrap();
    assert_eq!(bad_fields.code, "invalid_parameters");
    let host = json!({"dataset": "host", "query": QUERY, "latest": false});
    let host_search = quake.search(input(host.clone())).await.err().unwrap();
    let host_scroll = quake.scroll(input(host.clone())).await.err().unwrap();
    let mut host_aggregate = host;
    host_aggregate["aggregation_list"] = json!(["org"]);
    let host_aggregate = quake.aggregate(input(host_aggregate)).await.err().unwrap();
    for error in [host_search, host_scroll, host_aggregate] {
        assert_eq!(error.code, "invalid_parameters");
        assert!(error.message.contains("only to the service dataset"));
    }
    assert!(
        serde_json::from_value::<ScrollArgs>(
            json!({"cursor": "c", "query": QUERY, "dataset": "host"})
        )
        .is_err()
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn mcp_dispatch_returns_mocked_search_and_scroll_results() {
    use rmcp::ServiceExt;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::time::timeout;

    let mock = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v3/search/quake_service"))
        .respond_with(ok(
            json!([{"ip": "192.0.2.16", "port": 443}]),
            json!({"pagination": {"total": 1}}),
        ))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v3/scroll/quake_host"))
        .and(body_json(
            json!({"query": QUERY, "size": 1, "include": ["ip"]}),
        ))
        .respond_with(ok(
            json!([{"ip": "192.0.2.17"}]),
            json!({"pagination_id": "upstream-host", "total": 2}),
        ))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/v3/scroll/quake_host"))
        .and(body_json(
            json!({"query": QUERY, "size": 1, "include": ["ip"], "pagination_id": "upstream-host"}),
        ))
        .respond_with(ok(
            json!([{"ip": "192.0.2.18"}]),
            json!({"pagination_id": "upstream-host", "total": 2}),
        ))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v3/user/info"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&mock)
        .await;

    let quake = adapter(&mock, Duration::from_secs(5));
    let (server_io, client_io) = tokio::io::duplex(8192);
    let worker = tokio::spawn(async move {
        crate::mcp::QuakeMcp::new(quake)
            .serve(tokio::io::split(server_io))
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap();
    });
    let (reader, mut writer) = tokio::io::split(client_io);
    let mut reader = BufReader::new(reader).lines();
    let mut cursor = Value::Null;
    for (id, name, args) in [
        (2, "quake_account_info", json!({})),
        (
            3,
            "quake_search",
            json!({"dataset": "service", "query": QUERY}),
        ),
        (
            4,
            "quake_scroll",
            json!({"dataset": "host", "query": QUERY, "size": 1, "include": ["ip"]}),
        ),
        (5, "quake_scroll", Value::Null),
    ] {
        if id == 2 {
            let initialize = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "test", "version": "1"}
            }});
            writer
                .write_all(format!("{initialize}\n").as_bytes())
                .await
                .unwrap();
            let response = timeout(Duration::from_secs(5), reader.next_line())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert_eq!(
                input::<Value>(serde_json::from_str(&response).unwrap())["result"]["serverInfo"]["name"],
                "quake-mcp"
            );
            writer
                .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n")
                .await
                .unwrap();
        }
        let args = if id == 5 {
            json!({"cursor": cursor})
        } else {
            args
        };
        let call = json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": name, "arguments": args}});
        writer
            .write_all(format!("{call}\n").as_bytes())
            .await
            .unwrap();
        let line = timeout(Duration::from_secs(5), reader.next_line())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(response["id"], id, "{line}");
        let result = &response["result"];
        let payload = &result["structuredContent"];
        match id {
            2 => {
                assert_eq!(result["isError"], true, "{line}");
                assert_eq!(payload["error"]["code"], "authentication");
            }
            3 => assert_eq!(payload["data"]["items"][0]["ip"], "192.0.2.16"),
            4 => {
                assert_eq!(payload["data"]["items"][0]["ip"], "192.0.2.17");
                cursor = payload["data"]["next_cursor"].clone();
                assert!(cursor.is_string());
            }
            5 => assert_eq!(payload["data"]["items"][0]["ip"], "192.0.2.18"),
            _ => unreachable!(),
        }
    }
    writer.shutdown().await.unwrap();
    timeout(Duration::from_secs(5), worker)
        .await
        .unwrap()
        .unwrap();
}
