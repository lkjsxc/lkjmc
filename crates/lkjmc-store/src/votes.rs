use postgres::Client;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteLink {
    pub id: String,
    pub title_key: String,
    pub url: String,
    pub sort_order: i32,
}

pub fn upsert(
    client: &mut Client,
    id: &str,
    title_key: &str,
    url: &str,
    sort_order: i32,
) -> Result<(), StoreError> {
    client.execute(
        "insert into vote_links (id, title_key, url, sort_order)
         values ($1, $2, $3, $4)
         on conflict (id) do update set title_key = excluded.title_key,
         url = excluded.url,
         sort_order = excluded.sort_order,
         updated_at = now()",
        &[&id, &title_key, &url, &sort_order],
    )?;
    Ok(())
}

pub fn list(client: &mut Client) -> Result<Vec<VoteLink>, StoreError> {
    let rows = client.query(
        "select id, title_key, url, sort_order from vote_links order by sort_order, id",
        &[],
    )?;
    Ok(rows
        .into_iter()
        .map(|row| VoteLink {
            id: row.get(0),
            title_key: row.get(1),
            url: row.get(2),
            sort_order: row.get(3),
        })
        .collect())
}
