use crate::args::CliCommand;
use crate::args::{parse_lines, value_after};
use crate::error::CliError;

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [sub, uuid] if sub == "inspect" => Ok(CliCommand::PlayerInspect {
            player_uuid: uuid.clone(),
        }),
        [sub] if sub == "points-top" => Ok(CliCommand::PlayerPointsTop { limit: 10 }),
        [sub, flag, limit] if sub == "points-top" && flag == "--limit" => {
            Ok(CliCommand::PlayerPointsTop {
                limit: parse_lines(limit)?,
            })
        }
        [sub, uuid, name, source, flag, payload] if sub == "snapshot" && flag == "--payload" => {
            Ok(CliCommand::PlayerSnapshot {
                player_uuid: uuid.clone(),
                name: name.clone(),
                source: source.clone(),
                payload_path: payload.clone(),
            })
        }
        [sub, uuid, flag, snapshot] if sub == "restore" && flag == "--snapshot" => {
            Ok(CliCommand::PlayerRestore {
                player_uuid: uuid.clone(),
                snapshot_id: snapshot.clone(),
            })
        }
        _ => parse_flags(values),
    }
}

fn parse_flags(values: &[String]) -> Result<CliCommand, CliError> {
    if values.first().map(|value| value.as_str()) != Some("snapshot") || values.len() < 8 {
        return Err(CliError::message(usage()));
    }
    let mut player_uuid = values[1].clone();
    let mut name = String::new();
    let mut source = String::new();
    let mut payload_path = String::new();
    let mut index = 2;
    while index < values.len() {
        match values[index].as_str() {
            "--player" => player_uuid = value_after(values, index, "--player")?,
            "--name" => name = value_after(values, index, "--name")?,
            "--source" => source = value_after(values, index, "--source")?,
            "--payload" => payload_path = value_after(values, index, "--payload")?,
            _ => return Err(CliError::message(usage())),
        }
        index += 2;
    }
    if name.is_empty() || source.is_empty() || payload_path.is_empty() {
        return Err(CliError::message(usage()));
    }
    Ok(CliCommand::PlayerSnapshot {
        player_uuid,
        name,
        source,
        payload_path,
    })
}

fn usage() -> &'static str {
    "usage: lkjmc player inspect UUID | player points-top [--limit N] | player snapshot UUID --name NAME --source INSTANCE --payload PATH | player restore UUID --snapshot SNAPSHOT_ID"
}
