use std::time::Instant;

use postgres::Client;

pub fn work(client: &mut Client, schema: &str, id: &str, journal: bool) -> Result<(bool, u128), ()> {
    let started = Instant::now();
    let mut tx = client.transaction().map_err(|_| ())?;
    let inserted = tx.execute(
        &format!("INSERT INTO {schema}.operations VALUES ($1, 'requested') ON CONFLICT DO NOTHING"),
        &[&id],
    ).map_err(|_| ())?;
    if inserted == 1 && journal {
        tx.execute(&format!("INSERT INTO {schema}.journal VALUES ($1, 'pending')"), &[&id]).map_err(|_| ())?;
    }
    tx.commit().map_err(|_| ())?;
    if inserted == 0 { return Ok((false, started.elapsed().as_micros())); }
    let claimed = client.execute(
        &format!("INSERT INTO {schema}.effect_attempts VALUES ($1, 'running') ON CONFLICT DO NOTHING"),
        &[&id],
    ).map_err(|_| ())?;
    if claimed != 1 { return Err(()); }
    if !std::process::Command::new("true").status().map_err(|_| ())?.success() { return Err(()); }
    client.execute(&format!("INSERT INTO {schema}.effects VALUES ($1, 'completed')"), &[&id]).map_err(|_| ())?;
    client.execute(&format!("UPDATE {schema}.operations SET state='completed' WHERE request_id=$1"), &[&id]).map_err(|_| ())?;
    if journal {
        client.execute(&format!("UPDATE {schema}.journal SET state='completed' WHERE request_id=$1"), &[&id]).map_err(|_| ())?;
    }
    Ok((true, started.elapsed().as_micros()))
}
