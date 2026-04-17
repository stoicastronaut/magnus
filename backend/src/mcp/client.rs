use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::{
    transport::{ConfigureCommandExt, TokioChildProcess},
    ServiceExt,
};
use tokio::process::Command;

use super::McpServer;

pub type McpClient = RunningService<rmcp::RoleClient, ()>;

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

pub async fn connect_server(
    server: &McpServer,
) -> Result<McpClient, Box<dyn std::error::Error>> {
    let token = server.token.clone().unwrap_or_default();
    let env_key = server.env_key.clone().unwrap_or_default();
    let client = ()
        .serve(TokioChildProcess::new(
            Command::new(&server.command).configure(|cmd| {
                for arg in &server.args {
                    cmd.arg(arg);
                }
                if !token.is_empty() && !env_key.is_empty() {
                    cmd.env(&env_key, &token);
                }
            }),
        )?)
        .await?;

    Ok(client)
}

pub async fn list_tools(
    mcp_client: &McpClient,
) -> Result<Vec<rmcp::model::Tool>, Box<dyn std::error::Error>> {
    let resources = mcp_client.list_all_tools().await?;
    Ok(resources)
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
) -> Result<String, Box<dyn std::error::Error>> {
    let result = mcp_client
        .call_tool(CallToolRequestParams {
            meta: None,
            name: tool_name.to_string().into(),
            arguments: arguments.as_object().cloned(),
            task: None,
        })
        .await?;

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
