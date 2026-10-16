# ERPNext MCP Server in Rust

A blazingly fast, production-ready [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server for **ERPNext** and the **Frappe Framework**, written entirely in **Rust**.

This project provides a high-performance rewrite of the TypeScript implementation (`rakeshgangwar/erpnext-mcp-server`). By eliminating Node.js runtime overhead, it delivers:
- **Instant Cold Starts**: Sub-5ms startup times, ideal for on-demand MCP process spawning.
- **Minimal Footprint**: Native standalone binary running in `<10MB` RAM.
- **Zero Protocol Contamination**: Strict `stdin`/`stdout` JSON-RPC message framing, routing all logging exclusively to `stderr`.
- **Automatic URL Encoding**: Native percent-encoding across all DocTypes, document IDs, and report names (e.g. `Sales Invoice`, `ACC-SINV-.2026.-0001`).
- **Deep Frappe Error Diagnostics**: Proactively extracts and cleans human-readable error messages from nested Frappe payloads (`_server_messages`, `exception`, `message`), stripping unnecessary HTML tags.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────┐
│               LLM Client (Claude / Cursor / IDE)       │
└───────────────────────────┬────────────────────────────┘
                            │ JSON-RPC 2.0 (stdio)
                            ▼
┌────────────────────────────────────────────────────────┐
│                   erpnext-mcp (Rust)                   │
│                                                        │
│  ┌─────────────────┐             ┌──────────────────┐  │
│  │   main.rs       │             │   models.rs      │  │
│  │ (tracing:stderr)│             │ (schemars / serde│  │
│  └────────┬────────┘             └────────┬─────────┘  │
│           │                               │            │
│           ▼                               ▼            │
│  ┌──────────────────────────────────────────────────┐  │
│  │                  server.rs                       │  │
│  │         (rmcp #[tool_router] / 11 Tools)         │  │
│  └────────────────────────┬─────────────────────────┘  │
│                           │                            │
│                           ▼                            │
│  ┌──────────────────────────────────────────────────┐  │
│  │                  client.rs                       │  │
│  │  (reqwest async pool, URL encoding, auth headers,│  │
│  │        Frappe error envelope unwrapping)         │  │
│  └────────────────────────┬─────────────────────────┘  │
└───────────────────────────┼────────────────────────────┘
                            │ HTTPS / REST API
                            ▼
┌────────────────────────────────────────────────────────┐
│               ERPNext / Frappe Framework               │
└────────────────────────────────────────────────────────┘
```

---

## 11 Supported MCP Tools

| # | Tool Name | Description | Key Inputs |
|---|---|---|---|
| 1 | `get_doctypes` | Retrieve list of available DocTypes, optionally filtered by module. | `module: Option<String>` |
| 2 | `get_doctype_fields` | Get schema fields, definitions, and metadata for a specific DocType. | `doctype: String` |
| 3 | `get_documents` | Query list of documents for a DocType with filtering, field projection, and pagination. | `doctype: String`, `filters: Option<Value>`, `fields: Option<Vec<String>>`, `limit: Option<u32>`, `order_by: Option<String>` |
| 4 | `get_document` | Fetch a single document by name, including all child tables. | `doctype: String`, `name: String` |
| 5 | `create_document` | Create a new document in ERPNext with provided field values. | `doctype: String`, `data: Value` |
| 6 | `update_document` | Update fields of an existing document. | `doctype: String`, `name: String`, `data: Value` |
| 7 | `delete_document` | Permanently delete a document by DocType and name. | `doctype: String`, `name: String` |
| 8 | `submit_document` | Submit a submittable document (sets `docstatus` to 1). | `doctype: String`, `name: String` |
| 9 | `cancel_document` | Cancel a submitted document (sets `docstatus` to 2). | `doctype: String`, `name: String` |
| 10 | `call_method` | Execute any whitelisted Frappe Python server-side method. | `method: String`, `args: Option<Value>`, `http_method: Option<String>` |
| 11 | `run_report` | Run standard or script ERPNext reports with optional filters. | `report_name: String`, `filters: Option<Value>` |

---

## Environment Variables

Configure the server using environment variables or a local `.env` file in the working directory:

| Variable | Required | Description | Example |
|---|---|---|---|
| `ERPNEXT_URL` | **Yes** | Base URL of the Frappe / ERPNext instance | `https://erp.example.com` |
| `ERPNEXT_API_KEY` | No | User API Key generated from Frappe user profile | `3a1f8c...` |
| `ERPNEXT_API_SECRET` | No | User API Secret generated from Frappe user profile | `b94e21...` |

> When `ERPNEXT_API_KEY` and `ERPNEXT_API_SECRET` are both provided, the server attaches the header `Authorization: token <KEY>:<SECRET>` to all outbound Frappe requests.

---

## Installation & Setup

### 1. Build from Source
Ensure Rust (1.80+) and Cargo are installed:

```bash
# Clone the repository
git clone https://github.com/<YOUR_FOLDER>/ErpNext_MCP.git
cd ErpNext_MCP

# Build the release binary
cargo build --release
```

The resulting single binary executable is located at `target/release/erpnext-mcp` (or `target/release/erpnext-mcp.exe` on Windows).

### 2. Configure MCP Clients

#### Google Antigravity (Antigravity IDE / Antigravity 2.0 / AGY)

Antigravity natively integrates MCP servers over stdio with automatic tool discovery and system prompt injection.

##### Option A: Global Configuration via `mcp_config.json`
Add the server entry to your global Antigravity configuration file:
- **Windows**: `%USERPROFILE%\.gemini\config\mcp_config.json`
- **macOS / Linux**: `~/.gemini/config/mcp_config.json`

```json
{
  "mcpServers": {
    "erpnext": {
      "command": "C:\\<YOUR_FOLDER>\\ErpNext_MCP\\target\\release\\erpnext-mcp.exe",
      "args": [],
      "env": {
        "ERPNEXT_URL": "https://erp.yourdomain.com",
        "ERPNEXT_API_KEY": "your_api_key_here",
        "ERPNEXT_API_SECRET": "your_api_secret_here"
      }
    }
  }
}
```

> **Tip for Linux/macOS users**: Replace the Windows path with your binary path, e.g. `"/usr/local/bin/erpnext-mcp"` or `"/path/to/target/release/erpnext-mcp"`.

##### Option B: Antigravity UI
1. In the Antigravity chat panel, click **Additional Options (`...`)** at the top right.
2. Select **MCP Servers**.
3. Click **Add Server**:
   - **Name**: `erpnext`
   - **Transport**: `stdio`
   - **Command**: Path to `erpnext-mcp` executable
   - **Environment Variables**: Add `ERPNEXT_URL`, `ERPNEXT_API_KEY`, and `ERPNEXT_API_SECRET`.
4. Save and reload the session. All 11 ERPNext tools will appear automatically in the agent's toolset.

##### Option C: As an Antigravity Plugin
You can also package this server inside an Antigravity plugin under `plugins/erpnext/mcp_config.json`:
```json
{
  "mcpServers": {
    "erpnext": {
      "command": "${pluginDir}/bin/erpnext-mcp",
      "env": {
        "ERPNEXT_URL": "${env:ERPNEXT_URL}"
      }
    }
  }
}
```

---

#### Claude Desktop
Add to your Claude Desktop configuration file:
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "erpnext": {
      "command": "C:\\<YOUR_FOLDER>\\ErpNext_MCP\\target\\release\\erpnext-mcp.exe",
      "env": {
        "ERPNEXT_URL": "https://erp.yourdomain.com",
        "ERPNEXT_API_KEY": "your_api_key",
        "ERPNEXT_API_SECRET": "your_api_secret"
      }
    }
  }
}
```

---

#### Cursor / Windsurf / Other MCP Clients
Add to your project's `.mcp.json` or workspace settings:

```json
{
  "mcpServers": {
    "erpnext": {
      "command": "C:\\<YOUR_FOLDER>\\ErpNext_MCP\\target\\release\\erpnext-mcp.exe",
      "env": {
        "ERPNEXT_URL": "https://erp.yourdomain.com",
        "ERPNEXT_API_KEY": "your_api_key",
        "ERPNEXT_API_SECRET": "your_api_secret"
      }
    }
  }
}
```

---

## Running Integration Tests

The project includes a comprehensive suite of integration tests utilizing `wiremock` to test against mock Frappe endpoints without requiring a live ERPNext instance:

```bash
cargo test --test integration
```

Tested capabilities include:
- DocType listing with and without module filters
- Schema inspection with URL encoding of spaced names
- Filtering and pagination queries
- Document lifecycle (create, get, update, delete, submit, cancel)
- Whitelisted method execution (GET and POST)
- Script report generation
- Frappe nested `_server_messages` error extraction & HTML stripping
- Authentication header injection

---

## Codebase Structure

```text
ErpNext_MCP/
├── Cargo.toml          # Rust crate definitions and dependencies
├── README.md           # Documentation and usage guide
├── src/
│   ├── lib.rs          # Library root exposing client, models, and server
│   ├── main.rs         # Stdio transport, stderr tracing, and env loading
│   ├── client.rs       # Async Frappe REST client & error parsing logic
│   ├── server.rs       # MCP Server implementation & tool definitions
│   └── models.rs       # Schemars-annotated tool argument models
└── tests/
    └── integration.rs  # Automated integration test suite with wiremock
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
