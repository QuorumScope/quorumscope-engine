use quorumscope_api::ApiDoc;
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../openapi/openapi.json");
    std::fs::write(path, ApiDoc::openapi().to_pretty_json()?)?;
    Ok(())
}
