//! Fastly-specific configuration utilities.

use {
    self::backends::BackendsConfig,
    crate::error::FastlyConfigError,
    serde_derive::Deserialize,
    std::{collections::HashMap, convert::TryInto, fs, path::Path, str::FromStr, sync::Arc},
    toml::value::Table,
};

/// Unit tests for the [`FastlyConfig`] and [`TestingConfig`] types.
#[cfg(test)]
mod unit_tests;

/// Types and deserializers for backend configuration settings.
mod backends;
pub use self::backends::Backend;
pub type Backends = HashMap<String, Arc<Backend>>;

/// Fastly-specific configuration information.
///
/// This `struct` represents the fields and values in a Compute@Edge package's `fastly.toml`.
#[derive(Debug, Clone)]
pub struct FastlyConfig {
    name: String,
    description: String,
    authors: Vec<String>,
    language: String,
    local_server: LocalServerConfig,
}

impl FastlyConfig {
    /// Get a reference to the package name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get a reference to the package description.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Get a reference to the package authors.
    pub fn authors(&self) -> &[String] {
        self.authors.as_ref()
    }

    /// Get a reference to the package language.
    pub fn language(&self) -> &str {
        self.language.as_str()
    }

    /// Get the backend configuration.
    pub fn backends(&self) -> &Backends {
        &self.local_server.backends.0
    }

    /// Parse a `fastly.toml` file into a `FastlyConfig`.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, FastlyConfigError> {
        fs::read_to_string(path.as_ref())
            .map_err(Into::into)
            .and_then(Self::from_str)
    }

    /// Parse a string containing TOML data into a `FastlyConfig`.
    fn from_str(toml: impl AsRef<str>) -> Result<Self, FastlyConfigError> {
        toml::from_str::<'_, TomlFastlyConfig>(toml.as_ref())
            .map_err(Into::into)
            .and_then(TryInto::try_into)
    }
}

impl FromStr for FastlyConfig {
    type Err = FastlyConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

/// Internal deserializer used to read data from a `fastly.toml` file.
///
/// Once a TOML file has been read using [`toml::from_str`][from-str], this can be converted into
/// a [`FastlyConfig`][conf].
///
/// [conf]: struct.FastlyConfig.html
/// [fromt-str]: https://docs.rs/toml/latest/toml/de/fn.from_str.html
#[derive(Deserialize)]
struct TomlFastlyConfig {
    local_server: Option<RawLocalServerConfig>,
    // AJT 2020.03.10: the following fields are marked as optional because, for the time being,
    // we are not expecting to actually use the fastly.toml manifest, but instead use a separate
    // TOML file for backend configuration.
    //
    // See https://github.com/fastly/Viceroy/issues/109 for additional context.
    name: Option<String>,
    description: Option<String>,
    authors: Option<Vec<String>>,
    language: Option<String>,
}

impl TryInto<FastlyConfig> for TomlFastlyConfig {
    type Error = FastlyConfigError;
    fn try_into(self) -> Result<FastlyConfig, Self::Error> {
        let Self {
            name,
            description,
            authors,
            language,
            local_server,
        } = self;
        let local_server = local_server
            .map(TryInto::try_into)
            .transpose()?
            .unwrap_or_default();
        Ok(FastlyConfig {
            name: name.unwrap_or_else(String::new),
            description: description.unwrap_or_else(String::new),
            authors: authors.unwrap_or_else(Vec::new),
            language: language.unwrap_or_else(String::new),
            local_server,
        })
    }
}

/// Configuration settings used for tests.
///
/// This represents all of the `fastly.toml` fields whose keys begin with `testing`. Currently this
/// section of the manifest is only used for providing backend definitions, but additional fields
/// may be added in the future.
#[derive(Clone, Debug, Default)]
pub struct LocalServerConfig {
    backends: BackendsConfig,
}

/// Internal deserializer used to read the `[testing]` section of a `fastly.toml` file.
///
/// Once a TOML file has been read using [`toml::from_str`], this can be converted into
/// a [`LocalServerConfig`] with [`TryInto::try_into`].
#[derive(Deserialize)]
struct RawLocalServerConfig {
    backends: Table,
}

impl TryInto<LocalServerConfig> for RawLocalServerConfig {
    type Error = FastlyConfigError;
    fn try_into(self) -> Result<LocalServerConfig, Self::Error> {
        let Self { backends } = self;
        backends
            .try_into()
            .map(|backends| LocalServerConfig { backends })
    }
}
