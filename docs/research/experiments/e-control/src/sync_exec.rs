use std::thread;
use std::time::{Duration, Instant};

use postgres::{error::SqlState, Client, NoTls};

use crate::{jobs, sync_pool, sync_work, Stats, WARMUP};

pub fn baseline(url: &str, schema: &str, journal: bool) -> Result<Stats, ()> {
    let mut client = Client::connect(url, NoTls).map_err(|_| ())?;
    for id in jobs("warmup", WARMUP) { let _ = sync_work::work(&mut client, schema, &id, journal)?; }
    clear(&mut client, schema)?;
    let started = Instant::now();
    let mut stats = Stats { connections: 1, ..Stats::default() };
    for id in jobs("baseline", crate::REQUESTS) {
        let (effect, micros) = sync_work::work(&mut client, schema, &id, journal)?;
        stats.micros.push(micros);
        if effect { stats.effects += 1; } else { stats.duplicates += 1; }
    }
    stats.wall_micros = started.elapsed().as_micros();
    stats.deadline = deadline(&mut client, schema)?;
    stats.drained = true;
    Ok(stats)
}

pub fn bounded(url: &str, schema: &str, journal: bool) -> Result<Stats, ()> {
    let rejected = sync_pool::pressure(url, schema)?;
    let _ = sync_pool::pooled(url, schema, journal, jobs("warmup", WARMUP))?;
    let mut client = Client::connect(url, NoTls).map_err(|_| ())?;
    clear(&mut client, schema)?;
    let mut stats = sync_pool::pooled(url, schema, journal, jobs("bounded", crate::REQUESTS))?;
    stats.rejected = rejected;
    stats.connections = sync_pool::WORKERS;
    stats.deadline = deadline(&mut client, schema)?;
    Ok(stats)
}

fn clear(client: &mut Client, schema: &str) -> Result<(), ()> {
    client.batch_execute(&format!(
        "TRUNCATE {schema}.operations, {schema}.journal, {schema}.effect_attempts, {schema}.effects, {schema}.deadline"
    )).map_err(|_| ())
}

fn deadline(client: &mut Client, schema: &str) -> Result<&'static str, ()> {
    let cancel = client.cancel_token();
    let thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(2));
        cancel.cancel_query(NoTls).map_err(|_| ())
    });
    let result = client.batch_execute(&format!(
        "SELECT pg_sleep(0.05); INSERT INTO {schema}.deadline VALUES ('must-not-appear')"
    ));
    thread.join().map_err(|_| ())??;
    let absent = client.query_one(&format!("SELECT count(*) FROM {schema}.deadline"), &[])
        .map(|row| row.get::<_, i64>(0) == 0).map_err(|_| ())?;
    if matches!(result, Err(ref error) if error.code() == Some(&SqlState::QUERY_CANCELED)) && absent {
        Ok("cancelled-and-absent")
    } else { Err(()) }
}
