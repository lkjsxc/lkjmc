use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::async_exec::key;

pub(crate) async fn keyed() -> Result<(), ()> {
    let keys = Arc::new(Mutex::new(BTreeMap::new()));
    let same = Arc::new(AtomicUsize::new(0));
    let total = Arc::new(AtomicUsize::new(0));
    let maximum = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::new();
    for id in ["same", "same", "other"] {
        let keys = keys.clone();
        let same = same.clone();
        let total = total.clone();
        let maximum = maximum.clone();
        tasks.push(tokio::spawn(async move {
            let lock = key(&keys, id).await;
            let _guard = lock.lock().await;
            if id == "same" && same.fetch_add(1, Ordering::SeqCst) != 0 {
                return Err(());
            }
            maximum.fetch_max(total.fetch_add(1, Ordering::SeqCst) + 1, Ordering::SeqCst);
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            total.fetch_sub(1, Ordering::SeqCst);
            if id == "same" {
                same.fetch_sub(1, Ordering::SeqCst);
            }
            Ok(())
        }));
    }
    for task in tasks {
        task.await.map_err(|_| ())??;
    }
    if maximum.load(Ordering::SeqCst) >= 2 {
        Ok(())
    } else {
        Err(())
    }
}
