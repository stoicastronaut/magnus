# Keep MCP Connections Runtime-Only

Magnus persists Configured MCP Servers in App Data, but Connected MCP Server state is runtime-only and does not persist across app restarts. This avoids automatically launching or reconnecting external tool servers without an explicit product and security decision, even though users must reconnect saved MCP Servers after restarting the app. Future MCP auto-connect behavior should be designed separately before implementation.
