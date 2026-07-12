use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;

use crate::actor::{shard, ACTORS, QUEUE};

pub(crate) async fn keyed() -> Result<(), ()> {
    let same = Arc::new(AtomicUsize::new(0));
    let total = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let other = (b'a'..=b'z')
        .map(|b| format!("other-{}", b as char))
        .find(|id| shard(id) != shard("same"))
        .ok_or(())?;
    let mut routes = Vec::new();
    let mut workers = Vec::new();
    for _ in 0..ACTORS {
        let (sender, mut receiver) = mpsc::channel::<String>(QUEUE);
        routes.push(sender);
        let same = same.clone();
        let total = total.clone();
        let maximum = maximum.clone();
        workers.push(tokio::spawn(async move {
            while let Some(id) = receiver.recv().await {
                if id == "same" && same.fetch_add(1, Ordering::SeqCst) != 0 {
                    return Err(());
                }
                maximum.fetch_max(total.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis(5)).await;
                total.fetch_sub(1, Ordering::SeqCst);
                if id == "same" {
                    same.fetch_sub(1, Ordering::SeqCst);
                }
            }
            Ok(())
        }));
    }
    for id in ["same".into(), "same".into(), other] {
        routes[shard(&id)].try_send(id).map_err(|_| ())?;
    }
    drop(routes);
    for worker in workers {
        worker.await.map_err(|_| ())??;
    }
    if maximum.load(Ordering::SeqCst) >= 2 {
        Ok(())
    } else {
        Err(())
    }
}
