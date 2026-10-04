use quorumscope_domain::network::NetworkId;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum NetworkRegistryError {
    #[error("network '{name}' is registered with a different passphrase")]
    PassphraseMismatch { name: String },
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// Registers a network by name and returns its stable identifier.
/// Running this repeatedly with the same name and passphrase returns the same id.
pub async fn ensure_network(
    pool: &PgPool,
    name: &str,
    passphrase: &str,
) -> Result<NetworkId, NetworkRegistryError> {
    sqlx::query(
        "INSERT INTO networks (id, name, passphrase) VALUES ($1, $2, $3) ON CONFLICT (name) DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(name)
    .bind(passphrase)
    .execute(pool)
    .await?;
    let (id, stored): (Uuid, String) =
        sqlx::query_as("SELECT id, passphrase FROM networks WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await?;
    if stored != passphrase {
        return Err(NetworkRegistryError::PassphraseMismatch {
            name: name.to_string(),
        });
    }
    Ok(NetworkId::from_uuid(id))
}
