pub struct CursorManager {
    last_processed: u32,
}

impl CursorManager {
    pub fn new(start: u32) -> Self {
        Self { last_processed: start }
    }
    
    pub fn next_batch(&mut self, latest_network: u32, limit: u32) -> Option<std::ops::RangeInclusive<u32>> {
        if self.last_processed >= latest_network {
            return None;
        }
        
        let start = self.last_processed + 1;
        let end = std::cmp::min(start + limit - 1, latest_network);
        
        self.last_processed = end;
        Some(start..=end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_manager() {
        let mut manager = CursorManager::new(100);
        let batch = manager.next_batch(105, 10).unwrap();
        assert_eq!(batch, 101..=105);
        
        let batch2 = manager.next_batch(105, 10);
        assert!(batch2.is_none());
    }
}

use sqlx::PgPool;
use crate::error::IndexerError;

impl CursorManager {
    pub async fn load_from_db(pool: &PgPool, default_start: u32) -> Result<Self, IndexerError> {
        let record = sqlx::query(
            "SELECT MAX(ledger_sequence) as latest FROM incident_events"
        )
        .fetch_optional(pool)
        .await
        .map_err(|e| IndexerError::Pipeline(e.to_string()))?;

        let start = if let Some(row) = record {
            use sqlx::Row;
            let latest: Option<i64> = row.try_get("latest").unwrap_or(None);
            latest.map(|l| l as u32).unwrap_or(default_start)
        } else {
            default_start
        };

        Ok(Self::new(start))
    }
}
