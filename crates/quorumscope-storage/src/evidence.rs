use crate::error::StorageError;
use quorumscope_domain::evidence::{Evidence, EvidenceData, EvidenceKind};
use sqlx::PgPool;

pub struct EvidenceRepository {
    pool: PgPool,
}

impl EvidenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_evidence(&self, evidence: &Evidence) -> Result<(), StorageError> {
        let kind = match evidence.kind {
            EvidenceKind::PreflightTransaction => "preflight_transaction",
            EvidenceKind::BypassTransaction => "bypass_transaction",
            EvidenceKind::FreezeConfig => "freeze_config",
            EvidenceKind::LedgerEntry => "ledger_entry",
        };

        let data_json = match &evidence.data {
            EvidenceData::Xdr(xdr) => serde_json::json!({ "xdr": hex::encode(xdr) }),
            EvidenceData::Json(val) => val.clone(),
        };

        sqlx::query(
            r#"
            INSERT INTO evidence (id, incident_id, kind, data, canonical_xdr, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(evidence.id.as_uuid())
        .bind(evidence.incident_id.as_uuid())
        .bind(kind)
        .bind(data_json)
        .bind(evidence.canonical_xdr.as_deref())
        .bind(evidence.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
