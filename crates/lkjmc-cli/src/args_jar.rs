use crate::args::{value_after, CliCommand};
use crate::error::CliError;

pub fn parse(values: &[String]) -> Result<CliCommand, CliError> {
    match values {
        [sub] if sub == "list" => Ok(CliCommand::JarList),
        [sub, query] if sub == "inspect" => Ok(CliCommand::JarInspect {
            query: query.clone(),
        }),
        [sub, rest @ ..] if sub == "import" => parse_import(rest),
        _ => Err(CliError::message("usage: lkjmc jar list|inspect|import")),
    }
}

fn parse_import(values: &[String]) -> Result<CliCommand, CliError> {
    let mut kind = None;
    let mut name = None;
    let mut path = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--kind" => kind = Some(value_after(values, index, "--kind")?),
            "--name" => name = Some(value_after(values, index, "--name")?),
            "--path" => path = Some(value_after(values, index, "--path")?),
            other => {
                return Err(CliError::message(format!(
                    "unknown jar import flag: {other}"
                )))
            }
        }
        index += 2;
    }
    Ok(CliCommand::JarImport {
        kind: kind.ok_or_else(|| CliError::message("missing --kind"))?,
        name: name.ok_or_else(|| CliError::message("missing --name"))?,
        path: path.ok_or_else(|| CliError::message("missing --path"))?,
    })
}
