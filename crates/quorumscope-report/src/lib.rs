pub mod builder;
pub mod error;

#[cfg(test)]
mod tests {
    use super::builder::ReportBuilder;
    use quorumscope_domain::incident::{Incident, IncidentId};
    use quorumscope_domain::network::NetworkId;
    use std::fs;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_generate_pdf_fixture() {
        let builder = ReportBuilder::new("templates/report.typ");
        let incident = Incident {
            id: IncidentId(Uuid::new_v4()),
            network_id: NetworkId::new(),
            basis: "freeze_set_episode".to_string(),
            status: quorumscope_domain::incident::IncidentStatus::Active,
            opened_ledger: quorumscope_domain::ledger::LedgerSequence::new(100).unwrap(),
            opened_close_time: None,
            closed_ledger: None,
            closed_close_time: None,
        };

        let output_path = "fixtures_out.pdf";
        let _ = fs::remove_file(output_path); // ensure clean

        builder
            .generate_pdf(&incident, output_path)
            .await
            .expect("Typst failed");

        let meta = fs::metadata(output_path).expect("PDF should exist");
        assert!(meta.len() > 100, "PDF should be non-empty");

        fs::remove_file(output_path).unwrap();
    }
}
