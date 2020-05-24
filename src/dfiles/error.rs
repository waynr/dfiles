use dockworker;
use thiserror;
use which;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("not in entrypoint mode")]
    NotInEntrypointMode,

    #[error("missing entrypoint args")]
    MissingEntrypointArgs,

    #[error("could not find current binary")]
    CouldNotFindCurrentBinary(#[from] std::io::Error),

    #[error("failed to add file to archive")]
    FailedToAddFileToArchive { source: std::io::Error },

    #[error("could not identify user with uid `{0:?}`")]
    MissingUser(String),

    #[error("could not identify user with gid `{0:?}`")]
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

    #[error("failed to find binary")]
    WhichError(#[from] which::Error),
}
