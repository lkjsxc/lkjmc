use postgres::Client;
use uuid::Uuid;

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

pub fn reward(
    client: &mut Client,
    player_uuid: Uuid,
    player_name: &str,
    link_id: &str,
    points: i64,
    source: &str,
) -> Result<Uuid, StoreError> {
    crate::player::ensure_identity(client, player_uuid, Some(player_name))?;
    let reward_id = Uuid::new_v4();
    client.execute(
        "insert into player_vote_rewards
         (id, player_uuid, player_name, link_id, reward_points, source)
         values ($1, $2, $3, $4, $5, $6)",
        &[
            &reward_id,
            &player_uuid,
            &player_name,
            &link_id,
            &points,
            &source,
        ],
    )?;
    crate::points::grant(
        client,
        player_uuid,
        points,
        &format!("vote.reward:{link_id}"),
    )?;
    Ok(reward_id)
}
