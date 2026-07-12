use std::time::Instant;

use tokio::sync::{mpsc, oneshot, watch};

use crate::{actor_probe, effects, jobs, lab, Stats, WARMUP};

pub(crate) const ACTORS: usize = 4;
pub(crate) const QUEUE: usize = 8;

struct Message {
    id: String,
    done: oneshot::Sender<Result<(bool, u128), ()>>,
}
struct Pool {
    senders: Vec<mpsc::Sender<Message>>,
    gate: Option<watch::Sender<bool>>,
    workers: Vec<tokio::task::JoinHandle<Result<(), ()>>>,
}

pub async fn run(url: &str, schema: &str) -> Result<Stats, ()> {
    actor_probe::keyed().await?;
    let rejected = Pool::pressure(url, schema).await?;
    lab::reset(url, schema).await?;
    let pool = Pool::start(url, schema, false).await?;
    let _ = pool.execute(jobs("warmup", WARMUP)).await?;
    lab::reset(url, schema).await?;
    let started = Instant::now();
    let records = pool.execute(jobs("actor", crate::REQUESTS)).await?;
    let deadline = lab::deadline(url, schema).await?;
    pool.close().await?;
    let mut stats = Stats {
        wall_micros: started.elapsed().as_micros(),
        deadline,
        rejected,
        drained: true,
        connections: ACTORS,
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

impl Pool {
    async fn start(url: &str, schema: &str, held: bool) -> Result<Self, ()> {
        let (gate, wait) = watch::channel(!held);
        let mut senders = Vec::new();
        let mut workers = Vec::new();
        for _ in 0..ACTORS {
            let (sender, receiver) = mpsc::channel(QUEUE);
            senders.push(sender);
            workers.push(actor(url.into(), schema.into(), receiver, wait.clone()));
        }
        Ok(Self {
            senders,
            gate: held.then_some(gate),
            workers,
        })
    }

    async fn execute(&self, ids: Vec<String>) -> Result<Vec<(bool, u128)>, ()> {
        let mut waits = Vec::new();
        for id in ids {
            let (done, wait) = oneshot::channel();
            let mut message = Message {
                id: id.clone(),
                done,
            };
            let sender = &self.senders[shard(&id)];
            loop {
                match sender.try_send(message) {
                    Ok(()) => break,
                    Err(mpsc::error::TrySendError::Full(value)) => {
                        message = value;
                        tokio::task::yield_now().await;
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => return Err(()),
                }
            }
            waits.push(wait);
        }
        let mut records = Vec::new();
        for wait in waits {
            records.push(wait.await.map_err(|_| ())??);
        }
        Ok(records)
    }

    async fn pressure(url: &str, schema: &str) -> Result<usize, ()> {
        let pool = Self::start(url, schema, true).await?;
        let id = "actor-pressure";
        let sender = &pool.senders[shard(id)];
        let mut waits = Vec::new();
        for _ in 0..QUEUE {
            let (done, wait) = oneshot::channel();
            sender
                .try_send(Message {
                    id: id.into(),
                    done,
                })
                .map_err(|_| ())?;
            waits.push(wait);
        }
        let (done, _) = oneshot::channel();
        let rejected = usize::from(matches!(
            sender.try_send(Message {
                id: id.into(),
                done
            }),
            Err(mpsc::error::TrySendError::Full(_))
        ));
        pool.gate.as_ref().ok_or(())?.send(true).map_err(|_| ())?;
        for wait in waits {
            let _ = wait.await.map_err(|_| ())??;
        }
        pool.close().await?;
        Ok(rejected)
    }

    async fn close(self) -> Result<(), ()> {
        drop(self.senders);
        for worker in self.workers {
            worker.await.map_err(|_| ())??;
        }
        Ok(())
    }
}

fn actor(
    url: String,
    schema: String,
    mut receiver: mpsc::Receiver<Message>,
    mut gate: watch::Receiver<bool>,
) -> tokio::task::JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        while !*gate.borrow() {
            gate.changed().await.map_err(|_| ())?;
        }
        let (mut client, connection) = lab::connect(&url).await?;
        while let Some(message) = receiver.recv().await {
            let result = work(&mut client, &schema, &message.id).await;
            let _ = message.done.send(result);
        }
        connection.abort();
        Ok(())
    })
}

async fn work(
    client: &mut tokio_postgres::Client,
    schema: &str,
    id: &str,
) -> Result<(bool, u128), ()> {
    let started = Instant::now();
    let tx = client.transaction().await.map_err(|_| ())?;
    let inserted = tx
        .execute(
            &format!(
                "INSERT INTO {schema}.operations VALUES ($1, 'requested') ON CONFLICT DO NOTHING"
            ),
            &[&id],
        )
        .await
        .map_err(|_| ())?;
    if inserted == 1 {
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
    if !effects::run(client, schema, id).await? {
        return Err(());
    }
    client.batch_execute(&format!("UPDATE {schema}.operations SET state='completed' WHERE request_id='{id}'; UPDATE {schema}.journal SET state='completed' WHERE request_id='{id}'")).await.map_err(|_| ())?;
    Ok((true, started.elapsed().as_micros()))
}

pub(crate) fn shard(id: &str) -> usize {
    id.bytes().fold(0usize, |sum, byte| sum + byte as usize) % ACTORS
}
