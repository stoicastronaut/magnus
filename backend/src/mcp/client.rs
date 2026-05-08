use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::{
    ServiceExt,
    transport::{ConfigureCommandExt, TokioChildProcess},
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

    let _resources = client.list_all_tools().await?;

    Ok(())
}

pub async fn connect_server(server: &McpServer) -> Result<McpClient, McpError> {
    let client = ()
        .serve(
            TokioChildProcess::new(Command::new(&server.command).configure(
                |cmd| {
                    for arg in &server.args {
                        cmd.arg(arg);
                    }
                    if let Some((env_key, token)) = command_env(server) {
                        cmd.env(env_key, token);
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
            arguments: call_tool_arguments(&arguments),
            task: None,
        })
        .await
        .map_err(|e| McpError::CallTool {
            tool: tool_name.to_string(),
            source: e,
        })?;

    Ok(join_tool_content(&result.content))
}

fn command_env(server: &McpServer) -> Option<(&str, &str)> {
    match (server.env_key.as_deref(), server.token.as_deref()) {
        (Some(env_key), Some(token))
            if !env_key.is_empty() && !token.is_empty() =>
        {
            Some((env_key, token))
        }
        _ => None,
    }
}

fn call_tool_arguments(
    arguments: &serde_json::Value,
) -> Option<serde_json::Map<String, serde_json::Value>> {
    arguments.as_object().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    type Content = rmcp::model::Annotated<rmcp::model::RawContent>;

    fn text_content(text: &str) -> Content {
        serde_json::from_value(json!({"type": "text", "text": text})).unwrap()
    }

    fn server_with_env(
        token: Option<&str>,
        env_key: Option<&str>,
    ) -> McpServer {
        McpServer {
            name: "github".to_string(),
            display_name: "GitHub".to_string(),
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "server".to_string()],
            token: token.map(str::to_string),
            env_key: env_key.map(str::to_string),
        }
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

    #[test]
    fn command_env_requires_token_and_key() {
        assert_eq!(
            command_env(&server_with_env(Some("secret"), Some("TOKEN"))),
            Some(("TOKEN", "secret"))
        );
        assert_eq!(command_env(&server_with_env(None, Some("TOKEN"))), None);
        assert_eq!(command_env(&server_with_env(Some("secret"), None)), None);
        assert_eq!(
            command_env(&server_with_env(Some(""), Some("TOKEN"))),
            None
        );
        assert_eq!(
            command_env(&server_with_env(Some("secret"), Some(""))),
            None
        );
    }

    #[test]
    fn call_tool_arguments_accepts_only_json_objects() {
        assert_eq!(
            call_tool_arguments(&json!({"owner": "magnus", "limit": 2}))
                .unwrap(),
            serde_json::Map::from_iter([
                ("owner".to_string(), json!("magnus")),
                ("limit".to_string(), json!(2)),
            ])
        );
        assert_eq!(call_tool_arguments(&json!(null)), None);
        assert_eq!(call_tool_arguments(&json!(["not", "object"])), None);
    }
}
