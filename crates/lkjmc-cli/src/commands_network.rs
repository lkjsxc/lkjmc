mod dns;
mod minecraft_ping;

use std::collections::BTreeSet;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use lkjmc_core::network_diagnostics::{
    diagnose_network, AddressRecord, NetworkDiagnosticFacts, NetworkDiagnosticInput,
    ReachabilityCheck,
};

use crate::args_network::{NetworkCommand, NetworkDiagnoseOptions};
use crate::error::CliError;
use crate::format;

pub fn run(command: NetworkCommand, json_output: bool) -> Result<(), CliError> {
    match command {
        NetworkCommand::Diagnose(options) => diagnose(options, json_output),
    }
}

fn diagnose(options: NetworkDiagnoseOptions, json_output: bool) -> Result<(), CliError> {
    let input = NetworkDiagnosticInput {
        host: options.host.clone(),
        port: options.port,
        expected_address: options.expected_address.clone(),
        direct_address: options.direct_address.clone(),
    };
    let facts = gather(&options);
    let report = diagnose_network(input, facts);
    if json_output {
        return format::print_json(&serde_json::to_value(report)?);
    }
    println!("target: {}", report.effective_target);
    for finding in report.findings {
        println!(
            "{:?}: {:?}: {}",
            finding.severity, finding.code, finding.message
        );
    }
    for action in report.next_actions {
        println!("next: {action}");
    }
    Ok(())
}

fn gather(options: &NetworkDiagnoseOptions) -> NetworkDiagnosticFacts {
    let srv = dns::resolve_srv(&format!("_minecraft._tcp.{}", options.host));
    let mut address_records = addresses(&options.host, options.port);
    if let Some(first) = srv.first() {
        address_records.extend(addresses(&first.target, first.port));
    }
    let (target_host, target_port) = srv
        .first()
        .map(|record| (record.target.as_str(), record.port))
        .unwrap_or((options.host.as_str(), options.port));
    let target = format!("{target_host}:{target_port}");
    let mut tcp = vec![tcp_check(&target, target_host, target_port)];
    let mut status_ping = vec![status_check(
        &target,
        target_host,
        target_port,
        &options.host,
    )];
    if let Some(address) = &options.direct_address {
        let direct_target = format!("direct:{address}:{}", options.port);
        tcp.push(tcp_check(&direct_target, address, options.port));
        status_ping.push(status_check(&direct_target, address, options.port, address));
    }
    NetworkDiagnosticFacts {
        srv,
        addresses: address_records,
        tcp,
        status_ping,
    }
}

fn addresses(host: &str, port: u16) -> Vec<AddressRecord> {
    let mut seen = BTreeSet::new();
    let Ok(iter) = (host, port).to_socket_addrs() else {
        return Vec::new();
    };
    iter.filter_map(|addr| address_record(host, addr, &mut seen))
        .collect()
}

fn address_record(
    host: &str,
    addr: SocketAddr,
    seen: &mut BTreeSet<String>,
) -> Option<AddressRecord> {
    let ip = addr.ip().to_string();
    if !seen.insert(ip.clone()) {
        return None;
    }
    Some(AddressRecord {
        host: host.to_string(),
        address: ip,
        family: if addr.is_ipv4() { "ipv4" } else { "ipv6" }.to_string(),
    })
}

fn tcp_check(label: &str, host: &str, port: u16) -> ReachabilityCheck {
    match (host, port)
        .to_socket_addrs()
        .ok()
        .and_then(|mut values| values.next())
        .and_then(|addr| TcpStream::connect_timeout(&addr, Duration::from_secs(3)).ok())
    {
        Some(_) => ReachabilityCheck {
            target: label.to_string(),
            ok: true,
            message: "tcp connected".to_string(),
        },
        None => ReachabilityCheck {
            target: label.to_string(),
            ok: false,
            message: "tcp failed".to_string(),
        },
    }
}

fn status_check(label: &str, host: &str, port: u16, handshake_host: &str) -> ReachabilityCheck {
    match minecraft_ping::status_ping(host, port, handshake_host) {
        Ok(message) => ReachabilityCheck {
            target: label.to_string(),
            ok: true,
            message,
        },
        Err(error) => ReachabilityCheck {
            target: label.to_string(),
            ok: false,
            message: error,
        },
    }
}
