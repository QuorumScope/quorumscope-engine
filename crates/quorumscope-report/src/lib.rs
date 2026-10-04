pub mod builder;
pub mod data;
pub mod error;

#[cfg(test)]
mod tests {
    use super::builder::ReportBuilder;
    use super::data::{ReportData, ReportEvent, ReportKey, ReportSnapshot};

    /// Fixture values are synthetic. This verifies rendering, not network data.
    fn fixture(closed: Option<i64>, with_rows: bool) -> ReportData {
        ReportData {
            incident_id: "00000000-0000-4000-8000-000000000001".into(),
            network: "fixture-network".into(),
            basis: "freeze_set_episode".into(),
            status: "active".into(),
            opened_ledger: 100,
            closed_ledger: closed,
            keys: if with_rows {
                vec![ReportKey {
                    key_id: "ab".repeat(32),
                    kind: Some("account".into()),
                }]
            } else {
                vec![]
            },
            events: if with_rows {
                vec![ReportEvent {
                    ledger: 100,
                    kind: "freeze_change".into(),
                    evidence_ref: "config_snapshot:fixture".into(),
                }]
            } else {
                vec![]
            },
            snapshots: if with_rows {
                vec![ReportSnapshot {
                    ledger: 100,
                    frozen_accounts: 1,
                    frozen_trustlines: 0,
                    bypassed_transactions: 0,
                }]
            } else {
                vec![]
            },
        }
    }

    fn builder() -> ReportBuilder {
        if std::path::Path::new("../../bin/typst").exists() {
            ReportBuilder::with_typst("../../bin/typst")
        } else {
            ReportBuilder::new()
        }
    }

    #[test]
    fn renders_a_non_empty_pdf_with_rows_and_without() {
        for (closed, rows) in [(None, true), (Some(120), true), (Some(120), false)] {
            let out = std::env::temp_dir().join(format!("qs-report-{}.pdf", uuid::Uuid::new_v4()));
            let size = builder()
                .generate_pdf(&fixture(closed, rows), &out)
                .expect("typst should render the report");
            let bytes = std::fs::read(&out).unwrap();
            std::fs::remove_file(&out).unwrap();
            assert_eq!(bytes.len() as u64, size);
            assert!(bytes.starts_with(b"%PDF"), "output must be a PDF");
            assert!(size > 1000, "PDF should not be empty");
        }
    }

    #[test]
    fn missing_typst_binary_is_a_clear_error() {
        let out = std::env::temp_dir().join("qs-report-missing.pdf");
        let err = ReportBuilder::with_typst("/nonexistent/typst")
            .generate_pdf(&fixture(None, true), &out)
            .unwrap_err()
            .to_string();
        assert!(err.contains("could not run"));
    }
}
