use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_chat() -> Chat {
        Chat {
            id: "test-uuid".to_string(),
            name: "Test Chat".to_string(),
            messages: vec![],
            created_at: "01-01-25".to_string(),
        }
    }

    #[test]
    fn test_save_creates_file_with_correct_name() {
        let dir = tempdir().unwrap();
        let chat = make_chat();
        chat.save(dir.path()).unwrap();
        let expected = dir.path().join("01-01-25-test-uuid.json");
        assert!(expected.exists());
    }

    #[test]
    fn test_load_returns_correct_chat() {
        let dir = tempdir().unwrap();
        let chat = make_chat();
        chat.save(dir.path()).unwrap();
        let path = dir.path().join("01-01-25-test-uuid.json");
        let loaded = Chat::load(&path).unwrap();
        assert_eq!(loaded.id, chat.id);
        assert_eq!(loaded.name, chat.name);
        assert_eq!(loaded.created_at, chat.created_at);
    }

    #[test]
    fn test_delete_removes_files() {
        let dir = tempdir().unwrap();
        let chat = make_chat();
        chat.save(dir.path()).unwrap();
        chat.delete(dir.path()).unwrap();
        let expected = dir.path().join("01-01-25-uuid-test.json");
        assert!(!expected.exists());
    }

    #[test]
    fn test_delete_fails_on_missing_file() {
        let dir = tempdir().unwrap();
        let chat = make_chat();
        let result = chat.delete(dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_save_persists_messages() {
        let dir = tempdir().unwrap();
        let mut chat = make_chat();
        chat.messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello!".to_string(),
            },
            Message {
                role: "assistant".to_string(),
                content: "Hi!".to_string(),
            },
        ];
        chat.save(dir.path()).unwrap();
        let path = dir.path().join("01-01-25-test-uuid.json");
        let loaded = Chat::load(&path).unwrap();
        assert_eq!(loaded.messages.len(), 2);
        assert_eq!(loaded.messages[0].content, "Hello!");
        assert_eq!(loaded.messages[1].content, "Hi!");
    }
}
