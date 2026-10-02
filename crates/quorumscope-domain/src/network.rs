use std::fmt;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("invalid network passphrase for known network: expected '{expected}', got '{actual}'")]
    InvalidPassphrase { expected: String, actual: String },
    #[error("network name cannot be empty")]
    EmptyName,
    #[error("network passphrase cannot be empty")]
    EmptyPassphrase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NetworkId(Uuid);

impl NetworkId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for NetworkId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkName(String);

impl NetworkName {
    pub fn new(name: impl Into<String>) -> Result<Self, NetworkError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(NetworkError::EmptyName);
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPassphrase(String);

impl NetworkPassphrase {
    const MAINNET_PASSPHRASE: &'static str = "Public Global Stellar Network ; September 2015";
    const TESTNET_PASSPHRASE: &'static str = "Test SDF Network ; September 2015";
    const FUTURENET_PASSPHRASE: &'static str = "Test SDF Future Network ; Fall 2022";

    pub fn new(passphrase: impl Into<String>) -> Result<Self, NetworkError> {
        let passphrase = passphrase.into();
        if passphrase.trim().is_empty() {
            return Err(NetworkError::EmptyPassphrase);
        }
        Ok(Self(passphrase))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn validate_for_network(&self, network_name: &NetworkName) -> Result<(), NetworkError> {
        let expected = match network_name.as_str().to_lowercase().as_str() {
            "mainnet" => Some(Self::MAINNET_PASSPHRASE),
            "testnet" => Some(Self::TESTNET_PASSPHRASE),
            "futurenet" => Some(Self::FUTURENET_PASSPHRASE),
            _ => None,
        };

        if let Some(expected_passphrase) = expected
            && self.0 != expected_passphrase
        {
            return Err(NetworkError::InvalidPassphrase {
                expected: expected_passphrase.to_string(),
                actual: self.0.clone(),
            });
        }

        Ok(())
    }
}
