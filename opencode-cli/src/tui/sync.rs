use opencode_core::error::Result;
use opencode_core::session::Session;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::mpsc;

/// Lightweight session info sent from StateSync to the UI.
pub struct SessionListItem {
    pub id: String,
    pub title: String,
    pub updated: String,
}

pub struct StateSync {
    session_dir: PathBuf,
    tx: mpsc::UnboundedSender<Vec<SessionListItem>>,
    last_sync: std::time::Instant,
    sync_interval: std::time::Duration,
}

impl StateSync {
    pub fn new(
        session_dir: PathBuf,
        tx: mpsc::UnboundedSender<Vec<SessionListItem>>,
    ) -> Self {
        Self {
            session_dir,
            tx,
            last_sync: std::time::Instant::now(),
            sync_interval: std::time::Duration::from_secs(5),
        }
    }

    pub async fn sync_if_needed(&mut self) -> Result<()> {
        if self.last_sync.elapsed() >= self.sync_interval {
            self.sync().await?;
            self.last_sync = std::time::Instant::now();
        }
        Ok(())
    }

    /// Scans the session directory for session.json files, loads each session,
    /// and sends the list to the UI via the channel.
    async fn sync(&self) -> Result<()> {
        let mut list = Vec::new();
        let mut entries = match fs::read_dir(&self.session_dir).await {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let is_dir = entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false);
            if !is_dir {
                continue;
            }
            let session_file = path.join("session.json");
            if !session_file.exists() {
                continue;
            }
            let id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if id.is_empty() {
                continue;
            }
            match Session::load(&id, &self.session_dir).await {
                Ok(session) => {
                    list.push(SessionListItem {
                        id: session.id,
                        title: session.title,
                        updated: session.updated_at.to_rfc3339(),
                    });
                }
                Err(_) => continue,
            }
        }
        let _ = self.tx.send(list);
        Ok(())
    }

    pub fn set_sync_interval(&mut self, interval: std::time::Duration) {
        self.sync_interval = interval;
    }
}
