use crate::models::ApiKey;
use anyhow::{bail, Result};
use sqlx::{Pool, Postgres};

pub struct ApiKeyRepository<'a> {
  pool: &'a Pool<Postgres>,
}

impl<'a> ApiKeyRepository<'a> {
  pub fn new(pool: &'a Pool<Postgres>) -> Self {
    Self { pool }
  }

  pub async fn get_by_key(&self, key: &str) -> Result<ApiKey> {
    match sqlx::query_as!(
      ApiKey,
      "
SELECT *
FROM api_keys
WHERE key = $1
      ",
      key
    )
    .fetch_one(self.pool)
    .await
    {
      Ok(api_key) => Ok(api_key),
      Err(error) => match error {
        sqlx::Error::RowNotFound => bail!("Not found"),
        _ => bail!("Could not fetch api key"),
      },
    }
  }
}