#[tokio::main]
async fn main() -> anyhow::Result<()> {
    quorumscope_rpc::live_verify::run_live_verification().await
}
