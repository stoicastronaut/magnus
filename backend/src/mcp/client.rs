use rmcp::{ServiceExt, transport::{TokioChildProcess, ConfigureCommandExt}};
use rmcp::model::{ReadResourceRequestParams, CallToolRequestParams};
use rmcp::service::{RunningService};
use tokio::process::Command;

use super::McpServer;

pub type McpClient =  RunningService<rmcp::RoleClient, ()>;

pub async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    let client = ().serve(TokioChildProcess::new(Command::new("npx").configure(|cmd| {
        cmd.arg("-y").arg("@modelcontextprotocol/server-github");
        cmd.env("GITHUB_PERSONAL_ACCESS_TOKEN", "");
    }))?).await?;
    
    let resources = client.list_all_tools().await?;
    for resource in &resources {
        println!("{:#?}", resource);
    }
    
    Ok(())
}

pub async fn connect_server(server: &McpServer) -> Result<McpClient, Box<dyn std::error::Error>> {
    let token = server.token.clone().unwrap_or_default();
    let env_key = server.env_key.clone().unwrap_or_default();
    let client = ().serve(TokioChildProcess::new(Command::new(&server.command).configure(|cmd| {
        for arg in &server.args {
            cmd.arg(arg);
        }
        if !token.is_empty() && !env_key.is_empty() {
            cmd.env(&env_key, &token);
        }
    }))?).await?;

    Ok(client)
}

pub async fn list_tools(mcp_client: &McpClient) -> Result<Vec<rmcp::model::Tool>, Box<dyn std::error::Error>> {
    let resources = mcp_client.list_all_tools().await?;
    Ok(resources)
}

pub async fn call_tool(
    mcp_client: &McpClient,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {

    let result = mcp_client.call_tool(CallToolRequestParams {
        meta: None,
        name: tool_name.to_string().into(),
        arguments: arguments.as_object().cloned(),
        task: None,
    }).await?;

    let text = result.content
        .iter()
        .filter_map(|c| c.as_text())
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    Ok(text)
}

