use std::process::Stdio;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

#[tokio::test]
async fn binary_keeps_stdout_for_mcp_and_recovers_from_a_tool_error() {
    check_tool_session(false).await;
}

#[tokio::test]
async fn binary_exposes_the_same_tools_over_discover() {
    check_tool_session(true).await;
}

async fn check_tool_session(discover: bool) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_quake-mcp"))
        .env("QUAKE_TOKEN", "synthetic-token")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();

    let meta = json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientInfo": {"name": "test", "version": "1"},
        "io.modelcontextprotocol/clientCapabilities": {}
    });
    let messages = if discover {
        vec![
            json!({"jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": {"_meta": meta}}),
        ]
    } else {
        vec![
            json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "test", "version": "1"}
            }}),
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        ]
    };
    let mut calls = [
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
            "name": "quake_search", "arguments": {"dataset": "service", "query": " "}
        }}),
        json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {
            "name": "quake_search", "arguments": {"dataset": "service", "query": "domain:\"example.com\"", "latest": "false"}
        }}),
        json!({"jsonrpc": "2.0", "id": 5, "method": "tools/list", "params": {}}),
    ];
    if discover {
        for message in &mut calls {
            message["params"]["_meta"] = meta.clone();
        }
    }
    for message in messages.into_iter().chain(calls) {
        input
            .write_all(format!("{message}\n").as_bytes())
            .await
            .unwrap();
        let Some(id) = message.get("id") else {
            continue;
        };
        let line = timeout(Duration::from_secs(15), lines.next_line())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let response: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(&response["id"], id, "{line}");
        match id.as_u64() {
            Some(1) if discover => {
                let result = &response["result"];
                assert_eq!(
                    result["_meta"]["io.modelcontextprotocol/serverInfo"]["name"], "quake-mcp",
                    "{line}"
                );
                assert!(
                    result["supportedVersions"]
                        .as_array()
                        .unwrap()
                        .contains(&json!("2026-07-28"))
                );
            }
            Some(1) => assert_eq!(
                response["result"]["serverInfo"]["name"], "quake-mcp",
                "{line}"
            ),
            Some(2 | 5) => {
                let tools = response["result"]["tools"].as_array().expect(&line);
                let mut names: Vec<_> = tools
                    .iter()
                    .map(|tool| {
                        assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
                        assert_eq!(tool["outputSchema"]["type"], "object", "{tool}");
                        if matches!(
                            tool["name"].as_str(),
                            Some("quake_search" | "quake_scroll" | "quake_aggregate")
                        ) {
                            assert!(tool["inputSchema"].to_string().contains("\"latest\""));
                        }
                        tool["name"].as_str().unwrap()
                    })
                    .collect();
                names.sort_unstable();
                assert_eq!(
                    names,
                    [
                        "quake_account_info",
                        "quake_aggregate",
                        "quake_fields",
                        "quake_scroll",
                        "quake_search"
                    ]
                );
            }
            Some(3) => {
                assert_eq!(response["result"]["isError"], true, "{line}");
                assert_eq!(
                    response["result"]["structuredContent"]["error"]["code"],
                    "invalid_query"
                );
            }
            Some(4) => {
                assert!(
                    response["error"]["code"] == -32602 || response["result"]["isError"] == true,
                    "{line}"
                );
            }
            _ => unreachable!(),
        }
    }
    drop(input);
    assert!(
        timeout(Duration::from_secs(15), child.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
}

#[tokio::test]
async fn missing_token_fails_on_stderr() {
    let output = timeout(
        Duration::from_secs(15),
        Command::new(env!("CARGO_BIN_EXE_quake-mcp"))
            .env_remove("QUAKE_TOKEN")
            .kill_on_drop(true)
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("QUAKE_TOKEN"));
}
