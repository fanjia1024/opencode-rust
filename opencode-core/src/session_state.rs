use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Idle,
    Processing,
    WaitingForTool,
    WaitingForUser,
    Error,
    Completed,
}

pub struct SessionStateMachine {
    state: SessionState,
    retry_count: u32,
    max_retries: u32,
}

impl SessionStateMachine {
    pub fn new() -> Self {
        Self {
            state: SessionState::Idle,
            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn transition(&mut self, new_state: SessionState) -> Result<()> {
        match (self.state, new_state) {
            (SessionState::Idle, SessionState::Processing) => {
                self.state = new_state;
                Ok(())
            }
            (SessionState::Processing, SessionState::WaitingForTool) => {
                self.state = new_state;
                Ok(())
            }
            (SessionState::Processing, SessionState::WaitingForUser) => {
                self.state = new_state;
                Ok(())
            }
            (SessionState::Processing, SessionState::Completed) => {
                self.state = new_state;
                self.retry_count = 0;
                Ok(())
            }
            (SessionState::Processing, SessionState::Error) => {
                self.retry_count += 1;
                if self.retry_count >= self.max_retries {
                    self.state = SessionState::Error;
                    Err(crate::error::Error::Agent("Max retries exceeded".to_string()))
                } else {
                    self.state = SessionState::Idle;
                    Ok(())
                }
            }
            (SessionState::WaitingForTool, SessionState::Processing) => {
                self.state = new_state;
                Ok(())
            }
            (SessionState::WaitingForUser, SessionState::Processing) => {
                self.state = new_state;
                Ok(())
            }
            (SessionState::Error, SessionState::Idle) => {
                self.state = new_state;
                self.retry_count = 0;
                Ok(())
            }
            (SessionState::Completed, SessionState::Idle) => {
                self.state = new_state;
                Ok(())
            }
            _ => {
                Err(crate::error::Error::Agent(
                    format!("Invalid state transition: {:?} -> {:?}", self.state, new_state)
                ))
            }
        }
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    pub fn check_doom_loop(&self, recent_errors: usize) -> bool {
        recent_errors > 5 && self.retry_count >= self.max_retries
    }
}

impl Default for SessionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
