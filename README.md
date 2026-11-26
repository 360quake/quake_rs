# Quake MCP 与原始 Rust CLI

本地 Rust MCP Server，用于查询 360 Quake 已收录的主机和服务数据。支持偏移分页、逐页游标查询、聚合、字段发现和账号积分查询。接口参数与权限以 [360 Quake 帮助中心](https://quake.360.net/quake/#/help) 第 4 章为准。

## 构建与接入

- [面向人的安装与接入文档](docs/mcp-install.md) 提供 Windows、Linux 和 macOS 的安装命令、客户端配置和排错步骤。
- [面向 Agent 的安装手册](docs/mcp-install-agent.md) 提供可执行的安装步骤、配置模板和 MCP 验收流程。

安装当前稳定版 Rust。Linux 上构建还需要 OpenSSL 开发库和 `pkg-config`。

```sh
cargo build --release --locked
```

原始 `quake` CLI 保存在 [`legacy/quake_rs`](legacy/quake_rs/README.md)，源码和依赖取自提交 `5036473`。它使用原有凭证配置，可独立构建。

```sh
cargo build --release --locked --manifest-path legacy/quake_rs/Cargo.toml
```

从源码构建后，MCP 客户端的 `command` 使用 `target/release/quake-mcp` 的绝对路径。Windows 下文件名为 `quake-mcp.exe`。

## 工具

`dataset` 取 `service` 或 `host`。`query` 使用 Quake DSL。时间参数采用 UTC 的 `YYYY-MM-DD HH:MM:SS` 格式，是否可用由账号权限决定。

| 工具 | 参数与行为 |
| --- | --- |
| `quake_search` | `dataset`、`query`、可选的 `start`、`size`、`start_time`、`end_time`、`include`、`exclude`、`latest`。用响应的 `next_start` 继续偏移分页。 |
| `quake_scroll` | 首次调用传 `dataset`、`query` 和可选搜索参数，续页时只传 `cursor`。服务保存原始参数并逐页请求。句柄单次有效，最长 5 分钟，进程重启后失效。 |
| `quake_aggregate` | `dataset`、`query`、含一到两个维度的 `aggregation_list`，以及可选的 `size`、起止时间和 `latest`。结果为每个维度的桶。 |
| `quake_fields` | `dataset` 和 `purpose`。`search` 列出投影字段，`aggregation` 列出聚合维度。 |
| `quake_account_info` | 无参数。返回月度积分、永久积分和账号停用状态。 |

查询自有资产时，可以先取一小页，再根据结果决定是否继续。例如，以下参数用保留域名展示调用格式。

```json
{
  "dataset": "service",
  "query": "domain:\"example.com\"",
  "size": 5,
  "include": ["ip", "port", "service.name"]
}
```

搜索默认每页 10 条，上限 50 条。聚合每维默认 5 条，上限 100 条。这些是 MCP 的本地限制。省略 `include` 和 `exclude` 时，搜索采用基础会员可用的精简字段。

`latest` 适用于服务搜索、服务游标首调和服务聚合。设置 `false` 可查询历史记录。复现旧 CLI 时还需传入相同的时间窗口。省略时由 Quake 采用默认值，主机查询应省略该参数。

工具在 `structuredContent.data` 中返回机器可读结果。查询结果含 `items`、`count`、`total`、`total_relation`、`has_more` 和续页参数。`has_more: null` 表示上游信息不足，`total_relation: gte` 表示总量下界。游标页的 `has_more: true` 表示可以续页，下一页仍可能为空。游标终止页以 `count: 0` 和 `has_more: false` 表示结束，同时保留此前已知的查询总量。工具错误通过 `isError` 和 `structuredContent.error` 返回。

搜索和聚合可能消耗 Quake 积分。每次工具调用最多发出一个上游请求，服务同一时刻处理一个请求。超时可能已消耗积分，服务不自动重试已发送的请求。续页请求一旦开始使用句柄，该句柄即失效。

## 迁移和验证

本项目从 [360quake/quake_rs](https://github.com/360quake/quake_rs) 重构而来。原先的 `search`、`host` 和 `info` 工作流分别对应搜索或游标工具、主机数据集、账号工具。Agent 生成 DSL 并调用 MCP，Quake 凭证由 MCP 进程的环境提供。

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

测试使用本地 mock 和 MCP 子进程，不需要真实 Token。实际部署后可以用自己的 Token 和自有资产做小页查询、续页与聚合验证。

许可证见 [Apache-2.0](LICENSE)。
