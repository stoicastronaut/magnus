use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::{
    transport::{ConfigureCommandExt, TokioChildProcess},
    ServiceExt,
};
use thiserror::Error;
use tokio::process::Command;

use super::McpServer;

pub type McpClient = RunningService<rmcp::RoleClient, ()>;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("failed to spawn MCP server process: {0}")]
    ProcessSpawn(#[source] std::io::Error),
    #[error("failed to connect to MCP server: {0}")]
    Connect(#[source] Box<rmcp::service::ClientInitializeError>),
    #[error("failed to list tools: {0}")]
    ListTools(#[source] rmcp::ServiceError),
    #[error("failed to call tool '{tool}': {source}")]
    CallTool {
        tool: String,
        #[source]
        source: rmcp::ServiceError,
    },
}

pub async fn connect() -> Result<(), Box<dyn std::error::Error>> {
    let client = ()
        .serve(TokioChildProcess::new(Command::new("npx").configure(
            |cmd| {
                cmd.arg("-y").arg("@modelcontextprotocol/server-github");
                cmd.env("GITHUB_PERSONAL_ACCESS_TOKEN", "");
            },
        ))?)
        .await?;

    let resources = client.list_all_tools().await?;
    for resource in &resources {
        println!("{:#?}", resource);
    }

    Ok(())
}

pub async fn connect_server(server: &McpServer) -> Result<McpClient, McpError> {
    let token = server.token.clone().unwrap_or_default();
    let env_key = server.env_key.clone().unwrap_or_default();
    let client = ()
        .serve(
            TokioChildProcess::new(Command::new(&server.command).configure(
                |cmd| {
                    for arg in &server.args {
                        cmd.arg(arg);
                    }
                    if !token.is_empty() && !env_key.is_empty() {
                        cmd.env(&env_key, &token);
                    }
                },
            ))
            .map_err(McpError::ProcessSpawn)?,
        )
        .await
        .map_err(|e| McpError::Connect(Box::new(e)))?;

    Ok(client)
}

pub async fn list_tools(
    mcp_client: &McpClient,
) -> Result<Vec<rmcp::model::Tool>, McpError> {
    mcp_client
        .list_all_tools()
        .await
        .map_err(McpError::ListTools)
}

pub fn join_tool_content(
    content: &[rmcp::model::Annotated<rmcp::model::RawContent>],
) -> String {
    content
        .iter()
        .filter_map(|c| c.as_text())
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn call_tool(
    mcp_client: &McpClient,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<String, McpError> {
    let result = mcp_client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: tool_name.to_string().into(),
            arguments: arguments.as_object().cloned(),
            task: None,
        })
        .await
        .map_err(|e| McpError::CallTool {
            tool: tool_name.to_string(),
            source: e,
        })?;

    Ok(join_tool_content(&result.content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    type Content = rmcp::model::Annotated<rmcp::model::RawContent>;

    fn text_content(text: &str) -> Content {
        serde_json::from_value(json!({"type": "text", "text": text})).unwrap()
    }

    #[test]
    fn test_join_single_text_part() {
        let content = vec![text_content("hello")];
        assert_eq!(join_tool_content(&content), "hello");
    }

    #[test]
    fn test_join_multiple_text_parts() {
        let content = vec![text_content("line one"), text_content("line two")];
        assert_eq!(join_tool_content(&content), "line one\nline two");
    }

    #[test]
    fn test_join_empty_content() {
        assert_eq!(join_tool_content(&[]), "");
    }
}
