use anyhow::{Context, Result};
use reqwest;
use serde::{Deserialize, Serialize};

use crate::{app::App, prompt::SYSTEM_PROMPT};

const COMPLETION_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

pub async fn call_gpt(token: &str, app: &App) -> Result<String> {
    let prompt_text = app.prompt_editor_state.get_display_text();
    let tree = app.file_browser_state.get_entire_tree();
    let file_paths = app.file_browser_state.get_included_entries();
    let query =
        construct_query(prompt_text, &tree, &file_paths).context("Failed to construct query")?;

    let request_payload = Chat::new_from_query(&query);

    let client = reqwest::Client::new();
    let resp = client
        .post(COMPLETION_ENDPOINT)
        .bearer_auth(token)
        .json(&request_payload)
        .send()
        .await?;

    if resp.status() != 200 {
        println!("Error: {}", resp.status());
        println!("Error: {}", resp.text().await?);
        return Ok("".to_owned());
    }

    let resp_text = resp.text().await?;
    let completion: Completion = serde_json::from_str(&resp_text)?;
    let response_text = completion.choices.first().unwrap().message.content.clone();

    Ok(response_text)
}

fn construct_query(query: &str, tree: &str, file_paths: &[String]) -> Result<String> {
    // user query
    let mut query = format!("{}\n\n", query);

    // file system hierarchy
    query.push_str(format!("### File Tree:\n{}\n\n", tree).as_str());

    query.push_str("### File Contents:\n");
    for path in file_paths {
        let file_contents =
            std::fs::read_to_string(path).context(format!("Failed to read file: {}", path))?;
        query.push_str(&format!("```\n// {}\n{}\n```\n", path, file_contents));
    }
    Ok(query)
}

#[derive(Serialize, Deserialize)]
struct Chat {
    model: String,
    messages: Vec<Message>,
}

impl Chat {
    fn new_from_query(query: &str) -> Self {
        Chat {
            model: "gpt-4o-2024-11-20".to_owned(),
            messages: vec![
                Message {
                    role: Role::System,
                    content: SYSTEM_PROMPT.to_owned(),
                },
                Message {
                    role: Role::User,
                    content: query.to_owned(),
                },
            ],
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Completion {
    model: String,
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_new_from_query() {
        let query = "hi, how are you doing?";
        let chat = Chat::new_from_query(query);
        assert_eq!(chat.model, "gpt-4");
        assert_eq!(chat.messages.len(), 2);
        assert_eq!(chat.messages[0].role, Role::System);
        assert_eq!(chat.messages[0].content, SYSTEM_PROMPT);
        assert_eq!(chat.messages[1].role, Role::User);
        assert_eq!(chat.messages[1].content, query);
    }
}
