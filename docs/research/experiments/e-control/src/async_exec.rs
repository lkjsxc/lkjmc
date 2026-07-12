use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{mpsc, oneshot, Mutex};

use crate::async_pool::{Job, Keys, Pool, WORKERS};
use crate::{async_probe, effects, jobs, lab, Stats, WARMUP};

pub async fn run(url: &str, schema: &str, journal: bool, mixed: bool) -> Result<Stats, ()> {
    let probe_effects = if mixed {
        Some(Arc::new(effects::Pool::start(url, schema).await?))
    } else {
        None
    };
    let mut rejected = Pool::pressure(url, schema, journal, probe_effects.clone()).await?;
    if mixed {
        rejected += effects::Pool::pressure(url, schema).await?;
    }
    if let Some(pool) = probe_effects {
        Arc::try_unwrap(pool).map_err(|_| ())?.close().await?;
    }
    lab::reset(url, schema).await?;
    let effects = if mixed {
        Some(Arc::new(effects::Pool::start(url, schema).await?))
    } else {
        None
    };
    let pool = Pool::start(url, schema, journal, effects.clone()).await?;
    let _ = pool.execute(jobs("warmup", WARMUP)).await?;
    lab::reset(url, schema).await?;
    async_probe::keyed().await?;
    let started = Instant::now();
    let records = pool
        .execute(jobs(if mixed { "mixed" } else { "async" }, crate::REQUESTS))
        .await?;
    let deadline = lab::deadline(url, schema).await?;
    pool.close().await?;
    if let Some(pool) = effects {
        Arc::try_unwrap(pool).map_err(|_| ())?.close().await?;
    }
    let connections = WORKERS + usize::from(mixed) * effects::WORKERS;
    let mut stats = Stats {
        wall_micros: started.elapsed().as_micros(),
        deadline,
        rejected,
        drained: true,
        connections,
        ..Stats::default()
    };
    for (effect, micros) in records {
        stats.micros.push(micros);
        if effect {
            stats.effects += 1;
        } else {
            stats.duplicates += 1;
        }
    }
    Ok(stats)
}

pub(crate) fn worker(
    url: String,
    schema: String,
    journal: bool,
    keys: Keys,
    effects: Option<Arc<effects::Pool>>,
    mut receiver: mpsc::Receiver<Job>,
) -> tokio::task::JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        let (mut client, connection) = lab::connect(&url).await?;
        while let Some(job) = receiver.recv().await {
            let lock = key(&keys, &job.id).await;
            let guard = lock.lock().await;
            let result = work(
                &mut client,
                &schema,
                &job.id,
                journal,
                job.interrupt,
                effects.as_deref(),
            )
            .await;
            drop(guard);
            let _ = job.done.send(result);
        }
        connection.abort();
        Ok(())
    })
}

pub(crate) async fn key(
    keys: &Arc<Mutex<BTreeMap<String, Arc<Mutex<()>>>>>,
    id: &str,
) -> Arc<Mutex<()>> {
    keys.lock()
        .await
        .entry(id.into())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

async fn work(
    client: &mut tokio_postgres::Client,
    schema: &str,
    id: &str,
    journal: bool,
    interrupt: bool,
    pool: Option<&effects::Pool>,
) -> Result<(bool, u128), ()> {
    let started = Instant::now();
    let tx = client.transaction().await.map_err(|_| ())?;
    let inserted = tx.execute(&format!("INSERT INTO {schema}.operations (request_id, state) VALUES ($1, 'requested') ON CONFLICT DO NOTHING"), &[&id]).await.map_err(|_| ())?;
    if inserted == 1 && journal {
        tx.execute(
            &format!("INSERT INTO {schema}.journal VALUES ($1, 'pending')"),
            &[&id],
        )
        .await
        .map_err(|_| ())?;
    }
    tx.commit().await.map_err(|_| ())?;
    if inserted == 0 {
        return Ok((false, started.elapsed().as_micros()));
    }
    if interrupt {
        let pid = effects::interrupt_after_launch(client, schema, id).await?;
        println!("E-CONTROL fault=journal-interrupt-after-launch pid={pid} retry=withheld");
        return Ok((false, started.elapsed().as_micros()));
    }
    let effect = match pool {
        Some(pool) => loop {
            match pool.submit(id.into()) {
                effects::Submission::Accepted(wait) => break wait.await.map_err(|_| ())??,
                effects::Submission::Full => tokio::task::yield_now().await,
                effects::Submission::Closed => return Err(()),
            }
        },
        None => effects::run(client, schema, id).await?,
    };
    if !effect {
        return Err(());
    }
    client
        .execute(
            &format!("UPDATE {schema}.operations SET state = 'completed' WHERE request_id = $1"),
            &[&id],
        )
        .await
        .map_err(|_| ())?;
    if journal {
        client
            .execute(
                &format!("UPDATE {schema}.journal SET state = 'completed' WHERE request_id = $1"),
                &[&id],
            )
            .await
            .map_err(|_| ())?;
    }
    Ok((true, started.elapsed().as_micros()))
}

pub async fn journal_interruption(url: &str, schema: &str) -> Result<(), ()> {
    let pool = Pool::start(url, schema, true, None).await?;
    let (done, wait) = oneshot::channel();
    pool.sender
        .send(Job {
            id: "journal-interrupted-after-launch".into(),
            interrupt: true,
            done,
        })
        .await
        .map_err(|_| ())?;
    if wait.await.map_err(|_| ())??.0 {
        return Err(());
    }
    pool.close().await?;
    let counts = lab::counts(url, schema).await?;
    let (client, connection) = lab::connect(url).await?;
    client.batch_execute(&format!("DELETE FROM {schema}.effects WHERE request_id='journal-interrupted-after-launch'; DELETE FROM {schema}.effect_attempts WHERE request_id='journal-interrupted-after-launch'; DELETE FROM {schema}.journal WHERE request_id='journal-interrupted-after-launch'; DELETE FROM {schema}.operations WHERE request_id='journal-interrupted-after-launch'")) .await.map_err(|_| ())?;
    connection.abort();
    let expected = crate::REQUESTS as i64 + 1;
    if counts.operations == expected
        && counts.journal == expected
        && counts.attempts == expected
        && counts.effects == crate::REQUESTS as i64
    {
        Ok(())
    } else {
        Err(())
    }
}
