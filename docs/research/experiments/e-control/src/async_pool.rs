use std::collections::BTreeMap;
use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch, Mutex};

use crate::{async_exec, effects};

pub(crate) const WORKERS: usize = 4;
const QUEUE: usize = 8;
pub(crate) type Keys = Arc<Mutex<BTreeMap<String, Arc<Mutex<()>>>>>;

pub(crate) struct Job {
    pub(crate) id: String,
    pub(crate) interrupt: bool,
    pub(crate) done: oneshot::Sender<Result<(bool, u128), ()>>,
}

pub(crate) struct Pool {
    pub(crate) sender: mpsc::Sender<Job>,
    gate: Option<watch::Sender<bool>>,
    dispatcher: tokio::task::JoinHandle<Result<(), ()>>,
    workers: Vec<tokio::task::JoinHandle<Result<(), ()>>>,
}

impl Pool {
    pub(crate) async fn start(
        url: &str,
        schema: &str,
        journal: bool,
        effects: Option<Arc<effects::Pool>>,
    ) -> Result<Self, ()> {
        Self::with_gate(url, schema, journal, effects, false).await
    }

    async fn with_gate(
        url: &str,
        schema: &str,
        journal: bool,
        effects: Option<Arc<effects::Pool>>,
        held: bool,
    ) -> Result<Self, ()> {
        let (sender, mut receiver) = mpsc::channel(QUEUE);
        let (gate, mut wait) = watch::channel(!held);
        let keys = Arc::new(Mutex::new(BTreeMap::new()));
        let mut routes = Vec::new();
        let mut workers = Vec::new();
        for _ in 0..WORKERS {
            let (route, route_receiver) = mpsc::channel(1);
            routes.push(route);
            workers.push(async_exec::worker(
                url.into(),
                schema.into(),
                journal,
                keys.clone(),
                effects.clone(),
                route_receiver,
            ));
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

    pub(crate) async fn execute(&self, ids: Vec<String>) -> Result<Vec<(bool, u128)>, ()> {
        let mut waits = Vec::with_capacity(ids.len());
        for id in ids {
            let (done, wait) = oneshot::channel();
            let mut job = Job {
                id,
                interrupt: false,
                done,
            };
            loop {
                match self.sender.try_send(job) {
                    Ok(()) => break,
                    Err(mpsc::error::TrySendError::Full(value)) => {
                        job = value;
                        tokio::task::yield_now().await;
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => return Err(()),
                }
            }
            waits.push(wait);
        }
        let mut values = Vec::with_capacity(waits.len());
        for wait in waits {
            values.push(wait.await.map_err(|_| ())??);
        }
        Ok(values)
    }

    pub(crate) async fn pressure(
        url: &str,
        schema: &str,
        journal: bool,
        effects: Option<Arc<effects::Pool>>,
    ) -> Result<usize, ()> {
        let pool = Self::with_gate(url, schema, journal, effects, true).await?;
        let mut waits = Vec::new();
        for index in 0..QUEUE {
            let (done, wait) = oneshot::channel();
            pool.sender
                .try_send(Job {
                    id: format!("async-pressure-{index}"),
                    interrupt: false,
                    done,
                })
                .map_err(|_| ())?;
            waits.push(wait);
        }
        let (done, _) = oneshot::channel();
        let rejected = usize::from(matches!(
            pool.sender.try_send(Job {
                id: "async-overflow".into(),
                interrupt: false,
                done
            }),
            Err(mpsc::error::TrySendError::Full(_))
        ));
        pool.gate.as_ref().ok_or(())?.send(true).map_err(|_| ())?;
        for wait in waits {
            if !wait.await.map_err(|_| ())??.0 {
                return Err(());
            }
        }
        pool.close().await?;
        Ok(rejected)
    }

    pub(crate) async fn close(self) -> Result<(), ()> {
        drop(self.sender);
        self.dispatcher.await.map_err(|_| ())??;
        for worker in self.workers {
            worker.await.map_err(|_| ())??;
        }
        Ok(())
    }
}
