use postgres::Client;
use uuid::Uuid;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MailMessage {
    pub id: Uuid,
    pub sender_name: String,
    pub body: String,
    pub read: bool,
}

pub fn find_recipient(client: &mut Client, name: &str) -> Result<Option<Uuid>, StoreError> {
    let row = client.query_opt(
        "select player_uuid from player_identities where lower(current_name) = lower($1) limit 1",
        &[&name],
    )?;
    Ok(row.map(|row| row.get(0)))
}

pub fn send(
    client: &mut Client,
    id: Uuid,
    recipient_uuid: Uuid,
    sender_uuid: Uuid,
    sender_name: &str,
    body: &str,
) -> Result<(), StoreError> {
    client.execute(
        "insert into player_mail_messages (id, recipient_uuid, sender_uuid, sender_name, body)
         values ($1, $2, $3, $4, $5)",
        &[&id, &recipient_uuid, &sender_uuid, &sender_name, &body],
    )?;
    Ok(())
}

pub fn inbox(
    client: &mut Client,
    recipient_uuid: Uuid,
    limit: i64,
) -> Result<Vec<MailMessage>, StoreError> {
    let rows = client.query(
        "select id, sender_name, body, read_at is not null
         from player_mail_messages where recipient_uuid = $1
         order by created_at desc limit $2",
        &[&recipient_uuid, &limit],
    )?;
    Ok(rows.into_iter().map(message_from_row).collect())
}

pub fn read(
    client: &mut Client,
    recipient_uuid: Uuid,
    id: Uuid,
) -> Result<Option<MailMessage>, StoreError> {
    let row = client.query_opt(
        "update player_mail_messages set read_at = coalesce(read_at, now())
         where id = $1 and recipient_uuid = $2
         returning id, sender_name, body, true",
        &[&id, &recipient_uuid],
    )?;
    Ok(row.map(message_from_row))
}

fn message_from_row(row: postgres::Row) -> MailMessage {
    MailMessage {
        id: row.get(0),
        sender_name: row.get(1),
        body: row.get(2),
        read: row.get(3),
    }
}
