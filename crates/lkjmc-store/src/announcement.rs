use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Announcement {
    pub id: Uuid,
    pub actor_name: String,
    pub server_id: String,
    pub message: String,
}

pub fn create(
    client: &mut Client,
    id: Uuid,
    actor_name: &str,
    server_id: &str,
    message: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into announcements (id, actor_name, server_id, message)
         values ($1, $2, $3, $4)",
        &[&id, &actor_name, &server_id, &message],
    )?;
    Ok(())
}

pub fn recent(
    client: &mut Client,
    server_id: &str,
    limit: i64,
) -> Result<Vec<Announcement>, StoreError> {
    let rows = client.query(
        "select id, actor_name, server_id, message from announcements
         where server_id = $1 order by created_at desc limit $2",
        &[&server_id, &limit],
    )?;
    Ok(rows.into_iter().map(announcement_from_row).collect())
}

fn announcement_from_row(row: postgres::Row) -> Announcement {
    Announcement {
        id: row.get(0),
        actor_name: row.get(1),
        server_id: row.get(2),
        message: row.get(3),
    }
}
