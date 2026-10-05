//! Administrator operation history, with neutral active status and bounded reads.
use crate::{
    App,
    auth::Actor,
    error::{Error, Result},
};
use axum::{
    Json,
    extract::{Query, State},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Filter {
    #[default]
    Active,
    Failed,
    History,
}
impl Filter {
    fn states(self) -> &'static [&'static str] {
        match self {
            Self::Active => &["queued", "leased", "waiting"],
            Self::Failed => &["failed", "delivery_unknown"],
            Self::History => &["succeeded", "failed", "cancelled", "delivery_unknown"],
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationsQuery {
    #[serde(default)]
    pub filter: Filter,
    pub cursor: Option<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Cursor {
    actor: Uuid,
    filter: Filter,
    at: chrono::DateTime<chrono::Utc>,
    id: Uuid,
}
fn cursor(query: &OperationsQuery, actor: Uuid) -> Result<Option<Cursor>> {
    query
        .cursor
        .as_deref()
        .map(|encoded| {
            if encoded.len() > 512 {
                return Err(Error::invalid("text.invalid_operations_cursor"));
            }
            let cursor: Cursor = URL_SAFE_NO_PAD
                .decode(encoded)
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .ok_or_else(|| Error::invalid("text.invalid_operations_cursor"))?;
            if cursor.actor != actor || cursor.filter != query.filter {
                return Err(Error::invalid("text.operations_cursor_wrong_view"));
            }
            Ok(cursor)
        })
        .transpose()
}

pub async fn read(
    State(app): State<App>,
    actor: Actor,
    Query(query): Query<OperationsQuery>,
) -> Result<Json<Value>> {
    if !actor.admin {
        return Err(Error::forbidden());
    }
    let before = cursor(&query, actor.id)?;
    // One statement gives the list and its counts the same database snapshot.
    // Payloads and results stay behind the existing authorized job-detail read.
    let mut response: Value = sqlx::query_scalar(
        "WITH page AS (
            SELECT j.id,j.kind,j.server_id,s.name AS server_name,j.state,j.error,
                   j.progress,j.created_at,j.updated_at
            FROM jobs j LEFT JOIN servers s ON s.id=j.server_id
            WHERE j.kind NOT IN ('server.logs','server.files','server.file.read')
              AND j.state=ANY($1::text[])
              AND ($2::timestamptz IS NULL OR (j.created_at,j.id)<($2,$3::uuid))
            ORDER BY j.created_at DESC,j.id DESC LIMIT 26
         )
         SELECT jsonb_build_object(
            'operations',coalesce((SELECT jsonb_agg(to_jsonb(page) ORDER BY created_at DESC,id DESC) FROM page),'[]'::jsonb),
            'counts',jsonb_build_object(
                'active',count(*) FILTER(WHERE state IN ('queued','leased','waiting')),
                'failed',count(*) FILTER(WHERE state IN ('failed','delivery_unknown')),
                'history',count(*) FILTER(WHERE state IN ('succeeded','failed','cancelled','delivery_unknown'))
            )) FROM jobs WHERE kind NOT IN ('server.logs','server.files','server.file.read')",
    ).bind(query.filter.states())
        .bind(before.as_ref().map(|c| c.at))
        .bind(before.as_ref().map(|c| c.id))
        .fetch_one(&app.db).await?;
    let rows = response["operations"]
        .as_array_mut()
        .ok_or_else(|| Error::internal("Operation projection is not an array"))?;
    let more = rows.len() > 25;
    rows.truncate(25);
    let next = if more {
        rows.last()
            .map(|row| -> Result<String> {
                Ok(URL_SAFE_NO_PAD.encode(
                    serde_json::to_vec(&Cursor {
                        actor: actor.id,
                        filter: query.filter,
                        at: serde_json::from_value(row["created_at"].clone())
                            .map_err(Error::internal)?,
                        id: serde_json::from_value(row["id"].clone()).map_err(Error::internal)?,
                    })
                    .map_err(Error::internal)?,
                ))
            })
            .transpose()?
    } else {
        None
    };
    response["filter"] = json!(query.filter);
    response["next_cursor"] = json!(next);
    crate::system_message::project_system_content(&mut response);
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cursor_is_bounded_and_bound_to_account_and_filter() {
        let actor = Uuid::new_v4();
        let encoded = URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&Cursor {
                actor,
                filter: Filter::Failed,
                at: chrono::Utc::now(),
                id: Uuid::new_v4(),
            })
            .unwrap(),
        );
        let query = OperationsQuery {
            filter: Filter::Failed,
            cursor: Some(encoded.clone()),
        };
        assert!(cursor(&query, actor).unwrap().is_some());
        assert!(cursor(&query, Uuid::new_v4()).is_err());
        assert!(
            cursor(
                &OperationsQuery {
                    filter: Filter::History,
                    cursor: Some(encoded)
                },
                actor
            )
            .is_err()
        );
        for value in ["invalid".to_owned(), "x".repeat(513)] {
            assert!(
                cursor(
                    &OperationsQuery {
                        filter: Filter::Active,
                        cursor: Some(value)
                    },
                    actor
                )
                .is_err()
            );
        }
    }
}
