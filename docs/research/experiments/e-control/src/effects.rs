use std::process::Stdio;

use tokio::sync::{mpsc, oneshot, watch};

use crate::lab;

pub const WORKERS: usize = 4;
pub const QUEUE: usize = 8;

struct Job {
    id: String,
    done: oneshot::Sender<Result<bool, ()>>,
}

pub enum Submission {
    Accepted(oneshot::Receiver<Result<bool, ()>>),
    Full,
    Closed,
}

pub struct Pool {
    sender: mpsc::Sender<Job>,
    gate: Option<watch::Sender<bool>>,
    dispatcher: tokio::task::JoinHandle<Result<(), ()>>,
    workers: Vec<tokio::task::JoinHandle<Result<(), ()>>>,
}

impl Pool {
    pub async fn start(url: &str, schema: &str) -> Result<Self, ()> {
        Self::with_gate(url, schema, false).await
    }

    async fn with_gate(url: &str, schema: &str, held: bool) -> Result<Self, ()> {
        let (sender, mut receiver) = mpsc::channel(QUEUE);
        let (gate, mut wait) = watch::channel(!held);
        let mut routes = Vec::new();
        let mut workers = Vec::new();
        for _ in 0..WORKERS {
            let (route, route_receiver) = mpsc::channel(1);
            routes.push(route);
            workers.push(worker(url.into(), schema.into(), route_receiver));
        }
        let dispatcher = tokio::spawn(async move {
            while !*wait.borrow() {
                wait.changed().await.map_err(|_| ())?;
            }
            let mut next = 0;
            while let Some(job) = receiver.recv().await {
                routes[next % WORKERS].send(job).await.map_err(|_| ())?;
                next += 1;
            }
            Ok(())
        });
        Ok(Self {
            sender,
            gate: held.then_some(gate),
            dispatcher,
            workers,
        })
    }

    pub fn submit(&self, id: String) -> Submission {
        let (done, wait) = oneshot::channel();
        match self.sender.try_send(Job { id, done }) {
            Ok(()) => Submission::Accepted(wait),
            Err(mpsc::error::TrySendError::Full(_)) => Submission::Full,
            Err(mpsc::error::TrySendError::Closed(_)) => Submission::Closed,
        }
    }

    pub async fn pressure(url: &str, schema: &str) -> Result<usize, ()> {
        let pool = Self::with_gate(url, schema, true).await?;
        let mut waits = Vec::new();
        for index in 0..QUEUE {
            match pool.submit(format!("effect-pressure-{index}")) {
                Submission::Accepted(wait) => waits.push(wait),
                _ => return Err(()),
            }
        }
        let rejected = usize::from(matches!(
            pool.submit("effect-overflow".into()),
            Submission::Full
        ));
        pool.gate.as_ref().ok_or(())?.send(true).map_err(|_| ())?;
        for wait in waits {
            if !wait.await.map_err(|_| ())?? {
                return Err(());
            }
        }
        pool.close().await?;
        Ok(rejected)
    }

    pub async fn close(self) -> Result<(), ()> {
        drop(self.sender);
        self.dispatcher.await.map_err(|_| ())??;
        for worker in self.workers {
            worker.await.map_err(|_| ())??;
        }
        Ok(())
    }
}

fn worker(
    url: String,
    schema: String,
    mut receiver: mpsc::Receiver<Job>,
) -> tokio::task::JoinHandle<Result<(), ()>> {
    tokio::spawn(async move {
        let (mut client, connection) = lab::connect(&url).await?;
        while let Some(job) = receiver.recv().await {
            let result = run(&mut client, &schema, &job.id).await;
            let _ = job.done.send(result);
        }
        connection.abort();
        Ok(())
    })
}

pub async fn run(client: &mut tokio_postgres::Client, schema: &str, id: &str) -> Result<bool, ()> {
    let claimed = client.execute(&format!(
        "INSERT INTO {schema}.effect_attempts (request_id, state) VALUES ($1, 'running') ON CONFLICT DO NOTHING"
    ), &[&id]).await.map_err(|_| ())?;
    if claimed == 0 {
        return Ok(false);
    }
    let status = tokio::process::Command::new("true")
        .status()
        .await
        .map_err(|_| ())?;
    if !status.success() {
        return Err(());
    }
    client
        .execute(
            &format!("INSERT INTO {schema}.effects (request_id, state) VALUES ($1, 'completed')"),
            &[&id],
        )
        .await
        .map_err(|_| ())?;
    Ok(true)
}

pub async fn interrupt_after_launch(
    client: &mut tokio_postgres::Client,
    schema: &str,
    id: &str,
) -> Result<u32, ()> {
    let claimed = client.execute(&format!(
        "INSERT INTO {schema}.effect_attempts (request_id, state) VALUES ($1, 'running') ON CONFLICT DO NOTHING"
    ), &[&id]).await.map_err(|_| ())?;
    if claimed != 1 {
        return Err(());
    }
    let mut child = tokio::process::Command::new("sleep")
        .arg("10")
        .spawn()
        .map_err(|_| ())?;
    let pid = child.id().ok_or(())?;
    child.start_kill().map_err(|_| ())?;
    child.wait().await.map_err(|_| ())?;
    Ok(pid)
}

pub async fn interrupt_queued() -> Result<usize, ()> {
    let mut child = tokio::process::Command::new("sleep")
        .arg("10")
        .spawn()
        .map_err(|_| ())?;
    let pid = child.id().ok_or(())?;
    child.start_kill().map_err(|_| ())?;
    child.wait().await.map_err(|_| ())?;
    let survivors = usize::from(
        std::process::Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|_| ())?
            .success(),
    );
    println!(
        "E-CONTROL fault=interrupt-queued queued={} child_survivors={survivors}",
        QUEUE
    );
    Ok(survivors)
}
