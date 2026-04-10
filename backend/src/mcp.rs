use rmcp::{ServiceExt, transport::{TokioChildProcess, ConfigureCommandExt}};
use rmcp::model::{ReadResourceRequestParams};
use tokio::process::Command;

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
