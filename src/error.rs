//! Error types for the RustyWatch library API.
//!
//! Every fallible entry point in this crate returns [`Error`], so callers can
//! match on the failure instead of formatting a boxed trait object. The
//! [`Result`] alias defaults its error parameter to [`Error`]:
//!
//! ```no_run
//! use rustywatch::{Error, Result, Watcher};
//!
//! fn load() -> Result<Watcher> {
//!     match Watcher::from_config_file("rustywatch.yaml") {
//!         Ok(watcher) => Ok(watcher),
//!         Err(Error::ConfigRead { path, .. }) => {
//!             eprintln!("no config at {path}, falling back to defaults");
//!             Watcher::builder()
//!                 .workspace(rustywatch::Workspace::new(".").cmd("cargo run"))
//!                 .build()
//!         }
//!         Err(e) => Err(e),
//!     }
//! }
//! ```

use std::fmt;
use std::io;

/// A `Result` alias whose error type defaults to [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can go wrong while configuring or running RustyWatch.
///
/// The variants are `#[non_exhaustive]`: match with a trailing `_ =>` arm so
/// that new variants do not break your build.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The configuration file could not be read from disk.
    ConfigRead {
        /// Path that was attempted.
        path: String,
        /// Underlying I/O failure.
        source: io::Error,
    },
    /// The configuration file was read but is not valid RustyWatch YAML.
    ConfigParse {
        /// Path that was attempted.
        path: String,
        /// Underlying deserialization failure.
        source: serde_yaml::Error,
    },
    /// The configuration parsed but is semantically invalid (no workspaces, an
    /// empty `dir`, and similar).
    InvalidConfig(String),
    /// An ignore pattern could not be compiled into a glob matcher.
    Ignore {
        /// The offending pattern, when the failure can be attributed to one.
        pattern: Option<String>,
        /// Underlying `globset` failure.
        source: globset::Error,
    },
    /// The underlying filesystem watcher failed to start or to observe a path.
    Watch(notify::Error),
    /// A single workspace failed; the cause is preserved in `source`.
    Workspace {
        /// The workspace `dir` that failed.
        dir: String,
        /// Why it failed.
        source: Box<Error>,
    },
    /// A workspace task panicked or was cancelled before it finished.
    Task(tokio::task::JoinError),
}

impl Error {
    /// Wraps `source` as a failure attributed to the workspace at `dir`.
    pub fn workspace(dir: impl Into<String>, source: Error) -> Self {
        Error::Workspace {
            dir: dir.into(),
            source: Box::new(source),
        }
    }

    /// Builds an [`Error::InvalidConfig`] from any displayable message.
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Error::InvalidConfig(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ConfigRead { path, source } => {
                write!(f, "failed to read config file `{path}`: {source}")
            }
            Error::ConfigParse { path, source } => {
                write!(f, "failed to parse config file `{path}`: {source}")
            }
            Error::InvalidConfig(message) => write!(f, "invalid configuration: {message}"),
            Error::Ignore {
                pattern: Some(pattern),
                source,
            } => write!(f, "invalid ignore pattern `{pattern}`: {source}"),
            Error::Ignore {
                pattern: None,
                source,
            } => write!(f, "failed to compile ignore patterns: {source}"),
            Error::Watch(source) => write!(f, "filesystem watcher failed: {source}"),
            Error::Workspace { dir, source } => write!(f, "workspace `{dir}` failed: {source}"),
            Error::Task(source) => write!(f, "workspace task did not complete: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::ConfigRead { source, .. } => Some(source),
            Error::ConfigParse { source, .. } => Some(source),
            Error::InvalidConfig(_) => None,
            Error::Ignore { source, .. } => Some(source),
            Error::Watch(source) => Some(source),
            Error::Workspace { source, .. } => Some(source),
            Error::Task(source) => Some(source),
        }
    }
}

impl From<notify::Error> for Error {
    fn from(source: notify::Error) -> Self {
        Error::Watch(source)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(source: tokio::task::JoinError) -> Self {
        Error::Task(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn test_display_includes_source() {
        let err = Error::ConfigRead {
            path: "rustywatch.yaml".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "no such file"),
        };

        let rendered = err.to_string();
        assert!(rendered.contains("rustywatch.yaml"));
        assert!(rendered.contains("no such file"));
        assert!(err.source().is_some());
    }

    #[test]
    fn test_invalid_config_has_no_source() {
        let err = Error::invalid_config("workspaces must not be empty");
        assert_eq!(
            err.to_string(),
            "invalid configuration: workspaces must not be empty"
        );
        assert!(err.source().is_none());
    }

    #[test]
    fn test_workspace_wraps_cause() {
        let err = Error::workspace("./api", Error::invalid_config("dir must not be empty"));

        match &err {
            Error::Workspace { dir, source } => {
                assert_eq!(dir, "./api");
                assert!(matches!(**source, Error::InvalidConfig(_)));
            }
            other => panic!("expected Error::Workspace, got {other:?}"),
        }

        assert!(err.to_string().contains("workspace `./api` failed"));
    }

    #[test]
    fn test_notify_error_converts() {
        let err: Error = notify::Error::generic("boom").into();
        assert!(matches!(err, Error::Watch(_)));
    }

    fn config_parse_error() -> serde_yaml::Error {
        serde_yaml::from_str::<crate::Config>("workspaces: [").unwrap_err()
    }

    fn glob_error() -> globset::Error {
        globset::Glob::new("src/**/[").unwrap_err()
    }

    // Every variant must name what failed and keep the cause reachable, so a
    // caller can print either the one-liner or the full chain.
    #[test]
    fn test_every_variant_displays_and_exposes_its_source() {
        let cases: Vec<(Error, &str, bool)> = vec![
            (
                Error::ConfigRead {
                    path: "a.yaml".to_string(),
                    source: io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
                },
                "failed to read config file `a.yaml`",
                true,
            ),
            (
                Error::ConfigParse {
                    path: "b.yaml".to_string(),
                    source: config_parse_error(),
                },
                "failed to parse config file `b.yaml`",
                true,
            ),
            (
                Error::InvalidConfig("nope".to_string()),
                "invalid configuration: nope",
                false,
            ),
            (
                Error::Ignore {
                    pattern: Some("src/**/[".to_string()),
                    source: glob_error(),
                },
                "invalid ignore pattern `src/**/[`",
                true,
            ),
            (
                Error::Ignore {
                    pattern: None,
                    source: glob_error(),
                },
                "failed to compile ignore patterns",
                true,
            ),
            (
                Error::Watch(notify::Error::generic("boom")),
                "filesystem watcher failed",
                true,
            ),
            (
                Error::workspace("./api", Error::invalid_config("nope")),
                "workspace `./api` failed",
                true,
            ),
        ];

        for (err, expected, has_source) in cases {
            let rendered = err.to_string();
            assert!(
                rendered.contains(expected),
                "`{rendered}` should contain `{expected}`"
            );
            assert_eq!(
                err.source().is_some(),
                has_source,
                "unexpected source for `{rendered}`"
            );
            // `Debug` is part of the public contract via `unwrap`/`expect`.
            assert!(!format!("{err:?}").is_empty());
        }
    }

    #[tokio::test]
    async fn test_join_error_converts() {
        let handle = tokio::spawn(async { panic!("task blew up") });
        let join_error = handle.await.unwrap_err();

        let err: Error = join_error.into();
        assert!(matches!(err, Error::Task(_)));
        assert!(err.to_string().contains("workspace task did not complete"));
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<Error>();
    }

    #[test]
    fn test_error_boxes_into_std_error() {
        let boxed: Box<dyn std::error::Error> = Error::invalid_config("nope").into();
        assert!(boxed.to_string().contains("nope"));
    }
}
