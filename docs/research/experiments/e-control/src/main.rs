mod actor;
mod actor_probe;
mod async_exec;
mod async_pool;
mod async_probe;
mod effects;
mod lab;
mod sync_exec;
mod sync_pool;
mod sync_work;

use std::process::ExitCode;
use std::time::Instant;

pub const WARMUP: usize = 40;
pub const REQUESTS: usize = 200;
pub const DUPLICATES: usize = 16;

#[derive(Default)]
pub struct Stats {
    pub micros: Vec<u128>,
    pub wall_micros: u128,
    pub effects: usize,
    pub duplicates: usize,
    pub deadline: &'static str,
    pub rejected: usize,
    pub drained: bool,
    pub connections: usize,
    pub cleanup: usize,
}

impl Stats {
    pub fn report(&self, candidate: &str, repeat: usize, counts: lab::Counts) -> Result<(), ()> {
        let expected_rejections = if candidate == "mixed" {
            2
        } else {
            usize::from(candidate != "baseline")
        };
        if self.micros.is_empty()
            || counts.operations != REQUESTS as i64
            || counts.effects != REQUESTS as i64
            || counts.attempts != REQUESTS as i64
            || (candidate.contains("journal") && counts.journal != REQUESTS as i64)
            || self.effects != REQUESTS
            || self.duplicates != DUPLICATES
            || self.deadline != "cancelled-and-absent"
            || self.rejected != expected_rejections
            || !self.drained
            || self.cleanup != 0
        {
            return Err(());
        }
        let mut values = self.micros.clone();
        values.sort_unstable();
        let percentile = |n: usize| values[(values.len() * n).div_ceil(100) - 1];
        let rate = values.len() as u128 * 1_000_000 / self.wall_micros.max(1);
        println!(
            "E-CONTROL candidate={candidate} repeat={repeat} result=PASS requests={} p50_us={} p95_us={} throughput_ops_s={rate} effects={} duplicates={} operations={} journal={} attempts={} deadline={} rejected={} shutdown_drained={} child_survivors={} connections={}",
            values.len(), percentile(50), percentile(95), self.effects, self.duplicates,
            counts.operations, counts.journal, counts.attempts, self.deadline, self.rejected,
            self.drained, self.cleanup, self.connections
        );
        Ok(())
    }
}

pub fn jobs(prefix: &str, count: usize) -> Vec<String> {
    let mut ids = Vec::with_capacity(count + DUPLICATES);
    for index in 0..DUPLICATES {
        ids.push(format!("{prefix}-duplicate-{index}"));
        ids.push(format!("{prefix}-duplicate-{index}"));
    }
    ids.extend((DUPLICATES..count).map(|index| format!("{prefix}-{index}")));
    ids
}

async fn synchronous(
    url: String,
    schema: String,
    bounded: bool,
    journal: bool,
) -> Result<Stats, ()> {
    tokio::task::spawn_blocking(move || {
        if bounded {
            sync_exec::bounded(&url, &schema, journal)
        } else {
            sync_exec::baseline(&url, &schema, journal)
        }
    })
    .await
    .map_err(|_| ())?
}

async fn candidate(url: &str, schema: &str, name: &str) -> Result<Stats, ()> {
    match name {
        "baseline" => synchronous(url.into(), schema.into(), false, false).await,
        "bounded" => synchronous(url.into(), schema.into(), true, false).await,
        "async" => async_exec::run(url, schema, false, false).await,
        "mixed" => async_exec::run(url, schema, false, true).await,
        "bounded-journal" => synchronous(url.into(), schema.into(), true, true).await,
        "async-journal-actor" => actor::run(url, schema).await,
        _ => Err(()),
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let Some(url) = lab::disposable_url() else {
        println!(
            "E-CONTROL result=BLOCKED reason=disposable-loopback-or-compose-database-required"
        );
        return ExitCode::FAILURE;
    };
    if lab::wait_database(&url).await.is_err() {
        println!("E-CONTROL result=BLOCKED reason=database-health-timeout");
        return ExitCode::FAILURE;
    }
    let schema = lab::schema();
    if lab::setup(&url, &schema).await.is_err() {
        println!("E-CONTROL result=BLOCKED reason=database-setup-failed");
        return ExitCode::FAILURE;
    }
    let repeats = std::env::var("LKJMC_E_CONTROL_REPEATS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let started = Instant::now();
    let mut result =
        effects::interrupt_queued().await.and_then(
            |survivors| {
                if survivors == 0 {
                    Ok(())
                } else {
                    Err(())
                }
            },
        );
    for repeat in 1..=repeats {
        for name in [
            "baseline",
            "bounded",
            "async",
            "mixed",
            "bounded-journal",
            "async-journal-actor",
        ] {
            if result.is_err() || lab::reset(&url, &schema).await.is_err() {
                result = Err(());
                break;
            }
            let stats = candidate(&url, &schema, name).await;
            result = match stats {
                Ok(mut stats) => {
                    if name.contains("journal")
                        && async_exec::journal_interruption(&url, &schema)
                            .await
                            .is_err()
                    {
                        Err(())
                    } else {
                        stats.cleanup = 0;
                        lab::counts(&url, &schema)
                            .await
                            .and_then(|counts| stats.report(name, repeat, counts))
                    }
                }
                Err(()) => Err(()),
            };
            if result.is_err() {
                break;
            }
        }
    }
    let _ = lab::drop_schema(&url, &schema).await;
    if result.is_ok() {
        println!(
            "E-CONTROL result=PASS repeats={repeats} elapsed_ms={}",
            started.elapsed().as_millis()
        );
        ExitCode::SUCCESS
    } else {
        println!("E-CONTROL result=BLOCKED reason=candidate-invariant-failed");
        ExitCode::FAILURE
    }
}
