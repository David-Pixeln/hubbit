use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::HubbitResult, models::Session};

#[derive(Clone, Debug)]
pub struct SessionRepository {
  pool: PgPool,
}

impl SessionRepository {
  pub fn new(pool: PgPool) -> Self {
    Self { pool }
  }

  pub async fn is_device_active(&self, mac_addr: String) -> HubbitResult<bool> {
    match sqlx::query_as!(
      Session,
      "
SELECT *
FROM sessions
WHERE mac_address = $1 AND end_time + (15 * interval '1 minute') > NOW()
LIMIT 1
      ",
      mac_addr
    )
    .fetch_one(&self.pool)
    .await
    {
      Ok(_) => Ok(true),
      Err(sqlx::Error::RowNotFound) => Ok(false),
      Err(e) => Err(e.into()),
    }
  }

  pub async fn update_sessions(&self, devices: &[(Uuid, String)]) -> HubbitResult<()> {
    let macs = devices
      .iter()
      .map(|(_, mac)| mac.to_owned())
      .collect::<Vec<_>>();
    let active_sessions: Vec<Session> = sqlx::query_as!(
      Session,
      "
UPDATE sessions
SET end_time = NOW() + (5 * interval '1 minute')
WHERE mac_address = ANY($1) AND end_time + (15 * interval '1 minute') > NOW()
RETURNING *
      ",
      macs.as_slice()
    )
    .fetch_all(&self.pool)
    .await?;

    let inactive_devices = devices
      .iter()
      .filter(|&(user_id, _)| {
        !active_sessions
          .iter()
          .any(|active_sess| active_sess.user_id == *user_id)
      })
      .map(|user| user.to_owned())
      .collect::<Vec<_>>();
    let inactive_user_ids = inactive_devices
      .iter()
      .map(|(user_id, _)| user_id.to_owned())
      .collect::<Vec<_>>();
    let inactive_macs = inactive_devices
      .iter()
      .map(|(_, mac)| mac.to_owned())
      .collect::<Vec<_>>();

    sqlx::query!(
      "
INSERT INTO sessions (user_id, mac_address, start_time, end_time)
SELECT data.user_id, data.mac_address, NOW(), NOW() + (5 * interval '1 minute')
FROM UNNEST($1::uuid[], $2::CHAR(17)[]) as data(user_id, mac_address)
      ",
      &inactive_user_ids,
      &inactive_macs
    )
    .fetch_all(&self.pool)
    .await?;
    Ok(())
  }
}
