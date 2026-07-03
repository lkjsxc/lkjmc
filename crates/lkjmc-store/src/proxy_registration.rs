use postgres::Client;

use crate::error::StoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyRegistrationRecord {
    pub instance_id: String,
    pub connect_host: String,
    pub connect_port: i32,
    pub registered: bool,
    pub failure_reason: Option<String>,
    pub age_seconds: i64,
}

pub struct RegistrationReport<'a> {
    pub instance_id: &'a str,
    pub connect_host: &'a str,
    pub connect_port: i32,
    pub registered: bool,
    pub failure_reason: Option<&'a str>,
}

pub fn report(client: &mut Client, entries: &[RegistrationReport<'_>]) -> Result<(), StoreError> {
    let mut tx = client.transaction()?;
    for entry in entries {
        tx.execute(
            "insert into proxy_registrations
             (instance_id, connect_host, connect_port, registered, failure_reason, reported_at)
             values ($1, $2, $3, $4, $5, now())
             on conflict (instance_id) do update set
             connect_host = excluded.connect_host,
             connect_port = excluded.connect_port,
             registered = excluded.registered,
             failure_reason = excluded.failure_reason,
             reported_at = now()",
            &[
                &entry.instance_id,
                &entry.connect_host,
                &entry.connect_port,
                &entry.registered,
                &entry.failure_reason,
            ],
        )?;
    }
    tx.commit()?;
    Ok(())
}

pub fn get(
    client: &mut Client,
    instance_id: &str,
) -> Result<Option<ProxyRegistrationRecord>, StoreError> {
    let row = client.query_opt(
        "select instance_id, connect_host, connect_port, registered, failure_reason,
         extract(epoch from now() - reported_at)::bigint
         from proxy_registrations where instance_id = $1",
        &[&instance_id],
    )?;
    Ok(row.map(|row| ProxyRegistrationRecord {
        instance_id: row.get(0),
        connect_host: row.get(1),
        connect_port: row.get(2),
        registered: row.get(3),
        failure_reason: row.get(4),
        age_seconds: row.get(5),
    }))
}
