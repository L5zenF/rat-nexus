use rig::completion::Prompt;
use rig::agent::{Agent, AgentBuilder};
use crate::pages::tictactoe::Cell;
use anyhow::{Result, anyhow};
use rig::providers::openai;

pub struct CommentatorAgent {
    inner: Agent<openai::CompletionModel>,
}

impl CommentatorAgent {
    pub fn new(model: openai::CompletionModel) -> Self {
        let agent = AgentBuilder::new(model)
            .preamble("你是一个五子棋对局评论员。如果你赢了选手，你可以带有轻微的嘲讽，语气要幽默。
如果选手表现出色（虽然输了），则给予鼓励。
如果是平局，表现出遗憾。
如果选手赢了，表现出惊讶并赞美。
输入是对局的完整坐标记录和最终结果。
请用中文回复，字数控制在50字以内。")
            .build();
        
        Self { inner: agent }
    }

    pub async fn commentate(&self, history: &[(usize, usize, Cell)], result: &str) -> Result<String> {
        let mut history_str = String::new();
        for (i, (r, c, p)) in history.iter().enumerate() {
            let player_name = match p {
                Cell::Black => "玩家(黑)",
                Cell::White => "AI(白)",
                _ => "未知",
            };
            history_str.push_str(&format!("{}. {}: ({}, {})\n", i + 1, player_name, r + 1, c + 1));
        }

        let prompt = format!(
            "对局记录：\n{}\n最终结果：{}\n请发表你的评论：",
            history_str, result
        );

        self.inner.prompt(prompt).await
            .map_err(|e| anyhow!("Commentator prompt failed: {}", e))
    }
}
