use std::collections::BTreeSet;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::Instant;

use postgres::{Client, NoTls};

use crate::{sync_work, Stats};

pub const WORKERS: usize = 4;
const QUEUE: usize = 8;
type Keys = Arc<(Mutex<BTreeSet<String>>, Condvar)>;

pub fn pooled(url: &str, schema: &str, journal: bool, ids: Vec<String>) -> Result<Stats, ()> {
    let (sender, receiver) = mpsc::sync_channel::<String>(QUEUE);
    let receiver = Arc::new(Mutex::new(receiver));
    let records = Arc::new(Mutex::new(Vec::new()));
    let failed = Arc::new(Mutex::new(false));
    let keys = Arc::new((Mutex::new(BTreeSet::new()), Condvar::new()));
    let started = Instant::now();
    let mut workers = Vec::new();
    for _ in 0..WORKERS {
        workers.push(worker(
            url,
            schema,
            journal,
            receiver.clone(),
            records.clone(),
            failed.clone(),
            keys.clone(),
        ));
    }
    for id in ids {
        sender.send(id).map_err(|_| ())?;
    }
    drop(sender);
    for worker in workers {
        worker.join().map_err(|_| ())?;
    }
    if *failed.lock().map_err(|_| ())? {
        return Err(());
    }
    let records = records.lock().map_err(|_| ())?;
    let mut stats = Stats {
        wall_micros: started.elapsed().as_micros(),
        drained: true,
        ..Stats::default()
    };
    for &(effect, micros) in &*records {
        stats.micros.push(micros);
        if effect {
            stats.effects += 1;
        } else {
            stats.duplicates += 1;
        }
    }
    Ok(stats)
}

pub fn pressure(url: &str, schema: &str) -> Result<usize, ()> {
    let (sender, receiver) = mpsc::sync_channel::<String>(QUEUE);
    let receiver = Arc::new(Mutex::new(receiver));
    let gate = Arc::new(std::sync::Barrier::new(WORKERS + 1));
    let mut workers = Vec::new();
    for _ in 0..WORKERS {
        let queue = receiver.clone();
        let gate = gate.clone();
        let url = url.to_string();
        let schema = schema.to_string();
        workers.push(thread::spawn(move || {
            let Ok(mut client) = Client::connect(&url, NoTls) else {
                return false;
            };
            gate.wait();
            while let Some(id) = queue.lock().ok().and_then(|queue| queue.recv().ok()) {
                if sync_work::work(&mut client, &schema, &id, false).is_err() {
                    return false;
                }
            }
            true
        }));
    }
    for index in 0..QUEUE {
        sender
            .try_send(format!("pressure-{index}"))
            .map_err(|_| ())?;
    }
    let rejected = usize::from(sender.try_send("pressure-overflow".to_string()).is_err());
    gate.wait();
    drop(sender);
    for worker in workers {
        if !worker.join().map_err(|_| ())? {
            return Err(());
        }
    }
    Ok(rejected)
}

fn worker(
    url: &str,
    schema: &str,
    journal: bool,
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
    records: Arc<Mutex<Vec<(bool, u128)>>>,
    failed: Arc<Mutex<bool>>,
    keys: Keys,
) -> thread::JoinHandle<()> {
    let url = url.to_string();
    let schema = schema.to_string();
    thread::spawn(move || {
        let Ok(mut client) = Client::connect(&url, NoTls) else {
            fail(&failed);
            return;
        };
        loop {
            let id = match receiver.lock() {
                Ok(queue) => queue.recv(),
                Err(_) => {
                    fail(&failed);
                    return;
                }
            };
            let Ok(id) = id else { return };
            let result = keyed(&keys, &id, || {
                sync_work::work(&mut client, &schema, &id, journal)
            });
            match result {
                Ok(value) => match records.lock() {
                    Ok(mut values) => values.push(value),
                    Err(_) => {
                        fail(&failed);
                        return;
                    }
                },
                Err(()) => {
                    fail(&failed);
                    return;
                }
            }
        }
    })
}

fn keyed<T>(keys: &Keys, id: &str, action: impl FnOnce() -> Result<T, ()>) -> Result<T, ()> {
    let (lock, wake) = &**keys;
    let mut held = lock.lock().map_err(|_| ())?;
    while held.contains(id) {
        held = wake.wait(held).map_err(|_| ())?;
    }
    held.insert(id.to_string());
    drop(held);
    let result = action();
    if let Ok(mut held) = lock.lock() {
        held.remove(id);
        wake.notify_all();
    }
    result
}

fn fail(failed: &Mutex<bool>) {
    if let Ok(mut value) = failed.lock() {
        *value = true;
    }
}
