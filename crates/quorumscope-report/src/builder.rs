use crate::error::ReportError;
use quorumscope_domain::incident::Incident;
use std::process::Command;

pub struct ReportBuilder {
    template_path: String,
}

impl ReportBuilder {
    pub fn new(template_path: &str) -> Self {
        Self {
            template_path: template_path.to_string(),
        }
    }

    pub async fn generate_pdf(
        &self,
        incident: &Incident,
        output_path: &str,
    ) -> Result<(), ReportError> {
        let typst_bin = if std::path::Path::new("../../bin/typst").exists() {
            "../../bin/typst"
        } else {
            "typst"
        };

        let status = Command::new(typst_bin)
            .arg("compile")
            .arg("--input")
            .arg(format!("incident_id={}", incident.id.0))
            .arg("--input")
            .arg(format!("network={}", incident.network_id))
            .arg(&self.template_path)
            .arg(output_path)
            .status()
            .map_err(|e| ReportError::GenerationError(e.to_string()))?;

        if !status.success() {
            return Err(ReportError::GenerationError(
                "Typst compilation failed".into(),
            ));
        }

        Ok(())
    }
}
