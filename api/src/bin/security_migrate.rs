use harbour_chat_api::Config;
use harbour_chat_api::infrastructure::persistence::create_pool;
use harbour_chat_api::infrastructure::security::EnvelopeCrypto;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env();
    let crypto = EnvelopeCrypto::new(config.master_key_id.clone(), config.master_key_b64.clone())?;
    if !crypto.is_encryption_enabled() {
        anyhow::bail!("CHAT_MASTER_KEY_B64 is required for security migration");
    }

    let pool = create_pool(&config).await?;

    let rows = sqlx::query_as::<_, (String, String)>("SELECT id, content FROM messages")
        .fetch_all(&pool)
        .await?;
    let mut converted = 0usize;
    for (id, content) in rows {
        if content.starts_with("enc:v1:") {
            continue;
        }
        let encrypted = crypto.encrypt_text(&content)?;
        sqlx::query("UPDATE messages SET content = ? WHERE id = ?")
            .bind(encrypted)
            .bind(id)
            .execute(&pool)
            .await?;
        converted += 1;
    }

    let attachment_rows = sqlx::query_as::<_, (String,)>("SELECT storage_key FROM attachments")
        .fetch_all(&pool)
        .await?;
    let mut converted_files = 0usize;
    for (storage_key,) in attachment_rows {
        let path = config.data_dir.join("attachments").join(&storage_key);
        let Ok(bytes) = tokio::fs::read(&path).await else {
            continue;
        };
        if bytes.starts_with(b"encb:v1:") {
            continue;
        }
        let encrypted = crypto.encrypt_bytes(&bytes)?;
        tokio::fs::write(&path, encrypted).await?;
        converted_files += 1;
    }

    println!("Migrated {} messages and {} attachments to encrypted format", converted, converted_files);
    Ok(())
}
