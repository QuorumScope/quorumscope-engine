use quorumscope_domain::incident::Incident;
use crate::error::ReportError;
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

    pub async fn generate_pdf(&self, incident: &Incident, output_path: &str) -> Result<(), ReportError> {
        // Just a scaffold using typst CLI
        let status = Command::new("typst")
            .arg("compile")
            .arg(&self.template_path)
            .arg(output_path)
            .status()
            .map_err(|e| ReportError::GenerationError(e.to_string()))?;

        if !status.success() {
            return Err(ReportError::GenerationError("Typst compilation failed".into()));
        }

        Ok(())
    }
}
