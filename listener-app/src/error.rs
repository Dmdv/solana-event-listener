use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListenError {
    #[error("Config error")]
    Config,
}
