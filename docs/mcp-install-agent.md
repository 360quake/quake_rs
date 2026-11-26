# Quake MCP Agent 安装手册

在目标用户的机器上安装 `quake-mcp`，接入该用户选定的 `stdio` MCP 客户端，并验证工具发现和 Quake 认证。由用户自己操作时使用 [安装与接入说明](mcp-install.md)。

## 安装

1. 检查 `git --version` 和 `cargo +stable --version`。Windows 使用 MSVC 和 Windows SDK。Linux 需要 C 编译器、OpenSSL 开发库和 `pkg-config`。macOS 使用 Xcode Command Line Tools。 [Rust 安装说明](https://rust-lang.org/tools/install/)
2. 执行对应系统的安装命令并记录绝对路径。命令从 fork 的 `feat/quake-mcp` 分支构建仓库根目录的 `quake-mcp` 包。 [Cargo 安装说明](https://doc.rust-lang.org/cargo/commands/cargo-install.html)

Windows PowerShell：

```powershell
$installRoot = Join-Path $env:LOCALAPPDATA 'quake-mcp'
cargo +stable install --locked --git https://github.com/nikKrian/quake_rs.git --branch feat/quake-mcp --root $installRoot quake-mcp
if ($LASTEXITCODE -ne 0) { throw 'Quake MCP installation failed' }
$quakeMcp = Join-Path $installRoot 'bin/quake-mcp.exe'
Get-Item -LiteralPath $quakeMcp
```

Linux 或 macOS：

```bash
install_root="$HOME/.local/quake-mcp"
cargo +stable install --locked --git https://github.com/nikKrian/quake_rs.git --branch feat/quake-mcp --root "$install_root" quake-mcp &&
  quake_mcp="$install_root/bin/quake-mcp" &&
  test -x "$quake_mcp" &&
  printf '%s\n' "$quake_mcp"
```

## 接入客户端

在用户选定的客户端配置里合并 `quake` 条目。读取现有配置以保留其他 MCP 和设置。用绝对路径作为 `command`，参数列表为空。通过客户端的密钥配置或启动环境注入 `QUAKE_TOKEN`，安装报告中只记录注入方式和验证状态。

Codex CLI 的配置通常位于 `~/.codex/config.toml`。下面的路径是占位符。父进程应当已经具备 `QUAKE_TOKEN`，`env_vars` 将它转交给 MCP 子进程。配置后开启新的 Codex 会话。 [Codex MCP 配置](https://developers.openai.com/codex/mcp/)

```toml
[mcp_servers.quake]
command = '/absolute/path/to/quake-mcp'
env_vars = ['QUAKE_TOKEN']
```

采用 `mcpServers` JSON 的客户端可以用以下结构。把 `env` 值映射到客户端的密钥配置，或按客户端支持的方式从本地配置传入。

```json
{
  "mcpServers": {
    "quake": {
      "command": "/absolute/path/to/quake-mcp",
      "args": [],
      "env": {
        "QUAKE_TOKEN": "<your-quake-api-token>"
      }
    }
  }
}
```

## 验收

优先用配置后的客户端建立会话并读取工具列表。直接做协议探测时，将已安装的二进制启动为子进程并保持 stdin 打开。发送首行，等待 ID `1` 的响应，再依次发送初始化通知和工具列表请求。每条消息使用一行 JSON 并及时刷新。 [MCP 工具协议](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)

```jsonl
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"quake-install-check","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
```

确认返回的 `serverInfo.name` 是 `quake-mcp`，且工具列表包含 `quake_search`、`quake_scroll`、`quake_aggregate`、`quake_fields` 和 `quake_account_info`。仅做握手和工具发现时，可给临时探测进程注入一个测试值，工具列表无需访问 Quake API。

真实 Token 就绪后，用该 Token 重新启动或连接客户端，再调用账号工具。自动化安装到此即可验证认证。

```jsonl
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"quake_account_info","arguments":{}}}
```

成功结果的 `structuredContent.data` 含 `monthly_credits`、`persistent_credits`、`blocked`。工具失败时读取 `isError` 和 `structuredContent.error.code`。鉴权错误时检查密钥注入，刷新配置后重新开启会话。凭证暂缺时，将工具发现和实际认证分别报告。

## 首次资产查询

查询自有或已授权资产时，使用 `quake_fields` 发现投影或聚合字段。单页优先用 `quake_search`，用 `next_start` 替换 `start`。需要游标时先调用 `quake_scroll`，续页只传上一页返回的 `next_cursor`，并在 `has_more=false` 时结束。历史服务查询用 `latest=false` 和明确的 UTC 时间范围，主机查询省略 `latest`。资产搜索和聚合可能消耗积分，超时后先核对再重试。 [查询工具说明](../README.md#工具)

完成后报告仓库引用、实际二进制路径、客户端名称、工具发现状态和认证状态。
