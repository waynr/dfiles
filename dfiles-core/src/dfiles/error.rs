use dockworker;
use thiserror;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("must specify container command")]
    MustSpecifyContainerCommand,

    #[error("generic io error")]
    IOError(#[from] std::io::Error),

    #[error("generic fmt error")]
    FmtError(#[from] std::fmt::Error),

    #[error("could not find current binary")]
    CouldNotFindCurrentBinary(String),

    #[error("failed to add file to archive")]
    FailedToAddFileToArchive { source: std::io::Error },

    #[error("could not identify user `{0:?}`")]
    MissingUser(String),

    #[error("could not identify group `{0:?}`")]
    MissingGroup(String),

    #[error("invalid mount string `{0:?}`")]
    InvalidMount(String),

    #[error("invalid locale `{0}`")]
    InvalidLocale(String),

    #[error("invalid timezone `{0}`")]
    InvalidTimezone(String),

    #[error("could not identify directory")]
    MissingDirectory,

    #[error("directory")]
    DockerError(#[from] dockworker::errors::Error),

    #[error("log initialization error: {0}")]
    SetLoggerError(#[from] log::SetLoggerError),

    #[error("retrieving argument value: {0}")]
    MatchesError(#[from] clap::parser::MatchesError),

    #[error("failed to save config to file")]
    FailedToSaveConfig,

    #[error("failed to load config from file")]
    FailedToLoadConfig,

    #[error("local entrypoint path must exist")]
    LocalEntrypointPathMustExist,

    #[error("local entrypoint path must be a regular file")]
    LocalEntrypointPathMustBeARegularFile,

    #[error("local entyrpoint path must be executable")]
    LocalEntrypointPathMustBeExecutable,

    #[error("local entrypoint path must be absolute")]
    LocalEntrypointPathMustBeAbsolute,
}
