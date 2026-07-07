#!/usr/bin/env python3
"""Generate .mcp.json — only registers MCP servers that pass a readiness check."""
import base64
import glob
import http.client
import json
import os
import pathlib

servers: dict = {}

# --- o2-dev ---
zo_email = os.environ["ZO_ROOT_USER_EMAIL"]
zo_password = os.environ["ZO_ROOT_USER_PASSWORD"]
zo_port = os.environ.get("ZO_HTTP_PORT", "5080")

try:
    conn = http.client.HTTPConnection(f"localhost:{zo_port}", timeout=3)
    conn.request("GET", "/healthz")
    status = conn.getresponse().status
    conn.close()
    if status == 200:
        auth = base64.b64encode(f"{zo_email}:{zo_password}".encode()).decode()
        servers["o2-dev"] = {
            "type": "http",
            "url": f"http://localhost:{zo_port}/api/default/mcp",
            "headers": {"Authorization": f"Basic {auth}"},
        }
        print(f"o2-dev: registered (localhost:{zo_port})")
    else:
        print(f"o2-dev: skipped (healthz returned {status})")
except OSError:
    print(f"o2-dev: skipped (not reachable on localhost:{zo_port})")

# --- playwright ---
# Requires: playwright-mcp binary AND Chromium downloaded to the playwright cache volume
# (tepra-playwright-browsers volume → ~/.cache/ms-playwright) by `mise run setup:playwright`
hits = [
    p for p in pathlib.Path(".").rglob("node_modules/.bin/playwright-mcp")
    if ".pnpm" not in p.parts and "target" not in p.parts
]
chromium_cached = glob.glob(str(pathlib.Path.home() / ".cache/ms-playwright/chromium-*"))

if hits and chromium_cached:
    pnpm_dir = str(pathlib.Path(hits[0]).parent.parent.parent.resolve())
    servers["playwright"] = {
        "command": "mise",
        "args": ["exec", "--", "pnpm", "--dir", pnpm_dir, "exec", "playwright-mcp",
                 "--browser=chromium", "--headless"],
    }
    print(f"playwright: registered ({pnpm_dir})")
elif not hits:
    print("playwright: skipped (playwright-mcp not found — run `mise run setup:playwright`)")
else:
    print("playwright: skipped (Chromium not in cache — run `mise run setup:playwright`)")

pathlib.Path(".mcp.json").write_text(json.dumps({"mcpServers": servers}, indent="\t") + "\n")
print(".mcp.json generated")
