use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

/// Runtime settings that cannot be read from the database.
#[derive(Clone, Debug)]
pub struct ApiConfig {
    /// Highest protocol version this engine release has been verified against.
    /// Compatibility is reported as `unknown` when this is not configured.
    pub verified_protocol_max: Option<i32>,
    /// Seconds without a network observation before state is called stale.
    pub stale_after_secs: i64,
    /// Ledgers the indexer may trail the network before it is called behind.
    pub max_lag_ledgers: i64,
    /// Browser origins allowed to call the API. Empty means no CORS headers are sent.
    pub allowed_origins: Vec<String>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            verified_protocol_max: None,
            stale_after_secs: 300,
            max_lag_ledgers: 10,
            allowed_origins: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessStatus {
    /// The indexer has read the network recently and is not behind.
    Current,
    /// The indexer has read the network recently but trails the latest ledger.
    IndexingBehind,
    /// The indexer has not observed the network within the stale window.
    Stale,
    /// No indexed state exists for this network.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Compatibility {
    /// The network protocol is at or below the verified maximum.
    Verified,
    /// The network protocol is newer than the verified maximum.
    UnverifiedProtocol,
    /// The protocol version or the verified maximum is not known.
    Unknown,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct StateFreshness {
    pub status: FreshnessStatus,
    /// Ledger the stored freeze state was read at.
    pub source_ledger: Option<i64>,
    pub latest_network_ledger: Option<i64>,
    pub latest_indexed_ledger: Option<i64>,
    pub ingestion_lag_ledgers: Option<i64>,
    /// When the indexer last observed the network.
    pub observed_at: Option<String>,
    pub last_reconciled_ledger: Option<i64>,
    pub last_reconciled_at: Option<String>,
    pub current_protocol_version: Option<i32>,
    pub verified_protocol_max: Option<i32>,
    pub compatibility: Compatibility,
}

pub fn classify(
    config: &ApiConfig,
    indexed: Option<i64>,
    latest: Option<i64>,
    age_secs: Option<f64>,
) -> (FreshnessStatus, Option<i64>) {
    let lag = match (indexed, latest) {
        (Some(i), Some(l)) => Some((l - i).max(0)),
        _ => None,
    };
    let status = match (indexed, age_secs) {
        (None, _) => FreshnessStatus::Unknown,
        (Some(_), None) => FreshnessStatus::Unknown,
        (Some(_), Some(age)) if age > config.stale_after_secs as f64 => FreshnessStatus::Stale,
        (Some(_), Some(_)) if lag.is_some_and(|l| l > config.max_lag_ledgers) => {
            FreshnessStatus::IndexingBehind
        }
        _ => FreshnessStatus::Current,
    };
    (status, lag)
}

pub fn compatibility(config: &ApiConfig, protocol: Option<i32>) -> Compatibility {
    match (protocol, config.verified_protocol_max) {
        (Some(p), Some(max)) if p <= max => Compatibility::Verified,
        (Some(_), Some(_)) => Compatibility::UnverifiedProtocol,
        _ => Compatibility::Unknown,
    }
}

type Row = (
    Option<i64>,
    Option<i64>,
    Option<i32>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<f64>,
    Option<i64>,
    Option<chrono::DateTime<chrono::Utc>>,
);

pub async fn load(
    pool: &PgPool,
    config: &ApiConfig,
    network_id: Uuid,
) -> sqlx::Result<StateFreshness> {
    let row: Row = sqlx::query_as(
        "SELECT \
           (SELECT last_complete_ledger FROM ingestion_checkpoints WHERE network_id=$1 AND stream='cap77_config'), \
           o.latest_network_ledger, o.protocol_version, o.observed_at, \
           EXTRACT(EPOCH FROM (NOW() - o.observed_at))::float8, \
           o.last_reconciled_ledger, o.last_reconciled_at \
         FROM (SELECT $1::uuid AS id) n LEFT JOIN network_observations o ON o.network_id = n.id",
    )
    .bind(network_id)
    .fetch_one(pool)
    .await?;
    let (indexed, latest, protocol, observed_at, age, reconciled_ledger, reconciled_at) = row;
    let (status, lag) = classify(config, indexed, latest, age);
    Ok(StateFreshness {
        status,
        source_ledger: indexed,
        latest_network_ledger: latest,
        latest_indexed_ledger: indexed,
        ingestion_lag_ledgers: lag,
        observed_at: observed_at.map(|t| t.to_rfc3339()),
        last_reconciled_ledger: reconciled_ledger,
        last_reconciled_at: reconciled_at.map(|t| t.to_rfc3339()),
        current_protocol_version: protocol,
        verified_protocol_max: config.verified_protocol_max,
        compatibility: compatibility(config, protocol),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_separates_current_behind_stale_and_unknown() {
        let config = ApiConfig::default();
        assert_eq!(
            classify(&config, Some(100), Some(101), Some(5.0)).0,
            FreshnessStatus::Current
        );
        assert_eq!(
            classify(&config, Some(100), Some(150), Some(5.0)),
            (FreshnessStatus::IndexingBehind, Some(50))
        );
        assert_eq!(
            classify(&config, Some(100), Some(100), Some(900.0)).0,
            FreshnessStatus::Stale
        );
        assert_eq!(
            classify(&config, None, None, None).0,
            FreshnessStatus::Unknown
        );
        assert_eq!(
            classify(&config, Some(100), None, None).0,
            FreshnessStatus::Unknown
        );
    }

    #[test]
    fn compatibility_reports_newer_protocols_as_unverified() {
        let config = ApiConfig {
            verified_protocol_max: Some(25),
            ..ApiConfig::default()
        };
        assert_eq!(compatibility(&config, Some(25)), Compatibility::Verified);
        assert_eq!(
            compatibility(&config, Some(26)),
            Compatibility::UnverifiedProtocol
        );
        assert_eq!(compatibility(&config, None), Compatibility::Unknown);
        assert_eq!(
            compatibility(&ApiConfig::default(), Some(25)),
            Compatibility::Unknown
        );
    }
}
