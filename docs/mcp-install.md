# Quake MCP 安装与接入

用本地 Quake MCP 在支持标准输入输出（`stdio`）的客户端查询 360 Quake。安装需要 Git、Rust 稳定版和一个 Quake API Token。Agent 代装请使用 [Agent 安装手册](mcp-install-agent.md)。

## 1. 安装二进制

通过 rustup 安装 Rust 稳定版。Windows 选择 MSVC 工具链，并安装 Visual Studio 的 C++ 构建工具和 Windows SDK。Debian 或 Ubuntu 需要 `build-essential`、`pkg-config` 和 `libssl-dev`。macOS 需要 Xcode Command Line Tools。 [Rust 安装说明](https://rust-lang.org/tools/install/)

以下命令从仓库的 `feat/quake-mcp` 分支构建并安装到当前用户目录。最后一行会显示要填入客户端的程序路径。 [Cargo 安装说明](https://doc.rust-lang.org/cargo/commands/cargo-install.html)

### Windows PowerShell

```powershell
$installRoot = Join-Path $env:LOCALAPPDATA 'quake-mcp'
cargo +stable install --locked --git https://github.com/nikKrian/quake_rs.git --branch feat/quake-mcp --root $installRoot quake-mcp
if ($LASTEXITCODE -ne 0) { throw 'Quake MCP installation failed' }
Join-Path $installRoot 'bin/quake-mcp.exe'
```

### Linux 或 macOS

```bash
install_root="$HOME/.local/quake-mcp"
cargo +stable install --locked --git https://github.com/nikKrian/quake_rs.git --branch feat/quake-mcp --root "$install_root" quake-mcp &&
  printf '%s\n' "$install_root/bin/quake-mcp"
```

## 2. 配置 MCP 客户端

在 Quake 用户中心获取 API Token。客户端启动子进程时，需要为它提供 `QUAKE_TOKEN`。 [Quake 帮助中心](https://quake.360.net/quake/#/help)

### Codex CLI

将以下节加入用户级 `~/.codex/config.toml`。已有同名节时更新对应项。把 `command` 换成上一步得到的完整路径，Windows 可用 TOML 单引号保留路径中的反斜杠。 [Codex MCP 配置](https://developers.openai.com/codex/mcp/)

```toml
[mcp_servers.quake]
command = 'C:\Users\your-name\AppData\Local\quake-mcp\bin\quake-mcp.exe'
env_vars = ["QUAKE_TOKEN"]
```

再在用于启动 Codex 的终端设置 Token。Windows PowerShell：

```powershell
$quakeSecret = Read-Host -Prompt 'Quake API Token' -AsSecureString
$env:QUAKE_TOKEN = [System.Net.NetworkCredential]::new('', $quakeSecret).Password
Remove-Variable quakeSecret
codex mcp list
codex
```

Linux 或 macOS 的 Bash：

```bash
read -rs -p 'Quake API Token: ' QUAKE_TOKEN
printf '\n'
export QUAKE_TOKEN
codex mcp list
codex
```

`codex mcp list` 可以确认配置已被识别。实际连接和认证以新会话里的工具调用为准。

### 支持 `mcpServers` JSON 的客户端

在客户端指定的本地配置位置加入 `quake`，保留其他条目。配置支持密钥引用时，把客户端密钥映射到 `QUAKE_TOKEN`。以下是通过 `env` 传值的模板，在本机替换路径和 Token 占位符。

```json
{
  "mcpServers": {
    "quake": {
      "command": "/home/your-name/.local/quake-mcp/bin/quake-mcp",
      "args": [],
      "env": {
        "QUAKE_TOKEN": "<your-quake-api-token>"
      }
    }
  }
}
```

Windows 的 JSON 路径可以写成 `C:/Users/your-name/AppData/Local/quake-mcp/bin/quake-mcp.exe`。更新配置后重启客户端或开启新会话，让客户端启动 Quake MCP 子进程。

## 3. 验收和排错

在新会话中让客户端调用 `quake_account_info`，参数用 `{}`。成功时会返回账号停用状态、月度积分和永久积分。需要验证检索时，再用自有资产运行一次 `quake_search`，将 `size` 设为 `1`。搜索和聚合可能扣除积分。

| 现象 | 检查 |
| --- | --- |
| 找不到程序 | 确认 `command` 指向实际存在的绝对路径，Windows 保留 `.exe`。 |
| 启动后在终端等待 | 这是 `stdio` 服务等待 MCP 握手的表现。由 MCP 客户端启动它。 |
| 提示设置 `QUAKE_TOKEN` | 确认是启动客户端的进程提供了该变量。只在另一终端设置，已运行的客户端接收不到。 |
| `authentication` | 确认使用 Quake 用户中心当前有效的 Token。 |
| `permission_denied` 或 `quota_exhausted` | 用 `quake_account_info` 检查账号，再核对 Quake 权限和额度。 |
| `rate_limited` 或 `timeout` | 限流按 `retry_after_seconds` 等待。超时请求可能已经扣费，核对后再决定是否重试。 |

更新时重跑安装命令并重启客户端。从配置中移除 `quake` 后，可用 `cargo +stable uninstall --root <安装目录> quake-mcp` 卸载二进制。
