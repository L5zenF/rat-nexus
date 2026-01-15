use rig::completion::{Prompt, CompletionModel};
use rig::agent::{Agent, AgentBuilder};
use crate::pages::tictactoe::{Board, Cell, BOARD_SIZE};
use anyhow::{Result, anyhow};

/// A generic Gomoku agent that can work with any LLM provider supported by rig.
pub struct GomokuAgent<M: CompletionModel> {
    inner: Agent<M>,
}

impl<M: CompletionModel> GomokuAgent<M> {
    pub fn new(model: M) -> Self {
        let mut agent = AgentBuilder::new(model)
            .preamble("You are a Gomoku (Five in a Row) expert. 
You will be provided with a 15x15 board state. 
Empty cells are '.', Black (your opponent) is 'B', and White (you) is 'W'.
Your goal is to find the best move for White.
Respond ONLY with the coordinates in the format '(row, col)'.
For example: '(7, 7)'.")
            .build();
        
        agent.max_tokens = Some(10);
        
        Self { inner: agent }
    }

    /// Construct a string representation of the board for the LLM
    fn format_board(board: &Board) -> String {
        let mut board_str = String::new();
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                let cell = board.get(r, c);
                let ch = match cell {
                    Cell::Empty => '.',
                    Cell::Black => 'B',
                    Cell::White => 'W',
                };
                board_str.push(ch);
                board_str.push(' ');
            }
            board_str.push('\n');
        }
        board_str
    }

    pub async fn find_move(&self, board: &Board) -> Result<(usize, usize)> {
        let board_repr = Self::format_board(board);
        let response = self.inner.prompt(format!("Board: {},/nothink",board_repr)).await
            .map_err(|e| anyhow!("Agent prompt failed: {}", e))?;

        
        println!("GomokuAgent response: {}", response);
        self.parse_coordinates(&response)
    }

    fn parse_coordinates(&self, response: &str) -> Result<(usize, usize)> {
        let clean = response.trim().trim_matches(|c| c == '(' || c == ')');
        let parts: Vec<&str> = clean.split(',').collect();
        
        if parts.len() == 2 {
            let row = parts[0].trim().parse::<usize>()?;
            let col = parts[1].trim().parse::<usize>()?;
            if row < BOARD_SIZE && col < BOARD_SIZE {
                return Ok((row, col));
            }
        }
        
        Err(anyhow!("Invalid response from agent: {}", response))
    }
}
