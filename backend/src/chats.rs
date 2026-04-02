use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub messages: Vec<Message>,
    pub created_at: String,
}

impl Chat {
    pub fn save(&self, chats_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(chats_dir).map_err(|e| e.to_string())?;
        let filename = format!("{}-{}.json", self.created_at, self.id);
        let path = chats_dir.join(filename);
        let str_pretty =
            serde_json::to_string_pretty(&self).map_err(|e| e.to_string())?;
        fs::write(path, str_pretty).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Chat, String> {
        let json_str = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let json_pretty =
            serde_json::from_str(&json_str).map_err(|e| e.to_string())?;
        Ok(json_pretty)
    }

    pub fn delete(&self, chats_dir: &Path) -> Result<(), String> {
        let filename = format!("{}-{}.json", self.created_at, self.id);
        let file_path = chats_dir.join(filename);
        fs::remove_file(file_path).map_err(|e| e.to_string())?;
        Ok(())
    }
}
