use crate::data::ReportData;
use crate::error::ReportError;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

const TEMPLATE: &str = include_str!("../templates/report.typ");

pub struct ReportBuilder {
    typst: PathBuf,
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportBuilder {
    /// Uses the binary named by `QUORUMSCOPE_TYPST`, otherwise `typst` from `PATH`.
    pub fn new() -> Self {
        let typst = std::env::var_os("QUORUMSCOPE_TYPST")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("typst"));
        Self { typst }
    }

    pub fn with_typst(typst: impl Into<PathBuf>) -> Self {
        Self {
            typst: typst.into(),
        }
    }

    /// Renders the report to a PDF and returns the number of bytes written.
    pub fn generate_pdf(&self, data: &ReportData, output: &Path) -> Result<u64, ReportError> {
        let work = std::env::temp_dir().join(format!("quorumscope-report-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&work).map_err(|e| ReportError::GenerationError(e.to_string()))?;
        let result = self.render(data, &work, output);
        let _ = std::fs::remove_dir_all(&work);
        result
    }

    fn render(&self, data: &ReportData, work: &Path, output: &Path) -> Result<u64, ReportError> {
        let io = |e: std::io::Error| ReportError::GenerationError(e.to_string());
        std::fs::write(work.join("report.typ"), TEMPLATE).map_err(io)?;
        let json =
            serde_json::to_vec(data).map_err(|e| ReportError::GenerationError(e.to_string()))?;
        std::fs::write(work.join("data.json"), json).map_err(io)?;
        let out = Command::new(&self.typst)
            .arg("compile")
            .arg("--root")
            .arg(work)
            .arg(work.join("report.typ"))
            .arg(output)
            .output()
            .map_err(|e| {
                ReportError::GenerationError(format!(
                    "could not run {}: {e}. Install Typst or set QUORUMSCOPE_TYPST.",
                    self.typst.display()
                ))
            })?;
        if !out.status.success() {
            return Err(ReportError::GenerationError(format!(
                "Typst compilation failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        Ok(std::fs::metadata(output).map_err(io)?.len())
    }
}
