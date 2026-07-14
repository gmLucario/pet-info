use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub struct MagicLinkEntry {
    pub user_id: i64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum MagicLinkCommand {
    Create {
        token: String,
        user_id: i64,
        expires_in: Duration,
        resp: oneshot::Sender<Result<()>>,
    },
    Verify {
        token: String,
        resp: oneshot::Sender<Result<Option<i64>>>,
    },
}

#[derive(Clone)]
pub struct MagicLinkService {
    tx: mpsc::Sender<MagicLinkCommand>,
}

impl MagicLinkService {
    pub fn new(tx: mpsc::Sender<MagicLinkCommand>) -> Self {
        Self { tx }
    }

    pub async fn create_token(
        &self,
        token: &str,
        user_id: i64,
        expires_in: Duration,
    ) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(MagicLinkCommand::Create {
                token: token.to_string(),
                user_id,
                expires_in,
                resp: tx,
            })
            .await
            .context("Failed to send Create command to MagicLinkActor")?;
        rx.await
            .context("MagicLinkActor response channel dropped")?
    }

    pub async fn verify_token(&self, token: &str) -> Result<Option<i64>> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(MagicLinkCommand::Verify {
                token: token.to_string(),
                resp: tx,
            })
            .await
            .context("Failed to send Verify command to MagicLinkActor")?;
        rx.await
            .context("MagicLinkActor response channel dropped")?
    }
}

pub struct MagicLinkActor {
    dashmap: Arc<DashMap<String, MagicLinkEntry>>,
    rx: mpsc::Receiver<MagicLinkCommand>,
}

impl MagicLinkActor {
    pub fn spawn() -> (MagicLinkService, tokio::task::JoinHandle<()>) {
        let dashmap = Arc::new(DashMap::new());
        let (tx, rx) = mpsc::channel(100);
        let mut actor = Self { dashmap, rx };

        let handle = tokio::spawn(async move {
            actor.run().await;
        });

        (MagicLinkService::new(tx), handle)
    }

    async fn run(&mut self) {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            tokio::select! {
                cmd = self.rx.recv() => {
                    match cmd {
                        Some(MagicLinkCommand::Create { token, user_id, expires_in, resp }) => {
                            let entry = MagicLinkEntry {
                                user_id,
                                expires_at: Utc::now() + expires_in,
                            };
                            self.dashmap.insert(token, entry);
                            let _ = resp.send(Ok(()));
                        }
                        Some(MagicLinkCommand::Verify { token, resp }) => {
                            let res = match self.dashmap.get(&token) {
                                Some(entry) => {
                                    let entry_val = entry.clone();
                                    drop(entry); // Drop reference before modifying map
                                    // Lazy expiration & one-time use
                                    self.dashmap.remove(&token);

                                    Ok(if entry_val.expires_at > Utc::now() {
                                        Some(entry_val.user_id)
                                    } else {None})
                                }
                                None => Ok(None),
                            };

                            let _ = resp.send(res);
                        }
                        None => break, // Channel closed
                    }
                }
                _ = ticker.tick() => {
                    // Periodic cleanup of expired entries
                    let mut expired = Vec::new();
                    for entry in self.dashmap.iter() {
                        if entry.value().expires_at <= Utc::now() {
                            expired.push(entry.key().clone());
                        }
                    }
                    if !expired.is_empty() {
                        for k in expired {
                            self.dashmap.remove(&k);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_verify_token() {
        let (service, _handle) = MagicLinkActor::spawn();

        let token = "test_token_123".to_string();
        let user_id = 42;
        let expires_in = Duration::minutes(5);

        // Create token
        let create_res = service.create_token(&token, user_id, expires_in).await;
        assert!(create_res.is_ok());

        // Verify token
        let verify_res = service.verify_token(&token).await;
        assert!(verify_res.is_ok());
        assert_eq!(verify_res.unwrap(), Some(user_id));

        // Re-verifying should return None (one-time use)
        let re_verify_res = service.verify_token(&token).await;
        assert!(re_verify_res.is_ok());
        assert_eq!(re_verify_res.unwrap(), None);
    }

    #[tokio::test]
    async fn test_expired_token() {
        let (service, _handle) = MagicLinkActor::spawn();

        let token = "expired_token_123".to_string();
        let user_id = 99;

        // Create token that expires in -1 minutes (already expired)
        let expires_in = Duration::minutes(-1);
        let create_res = service.create_token(&token, user_id, expires_in).await;
        assert!(create_res.is_ok());

        // Verify token should return None due to lazy expiration
        let verify_res = service.verify_token(&token).await;
        assert!(verify_res.is_ok());
        assert_eq!(verify_res.unwrap(), None);
    }
}
