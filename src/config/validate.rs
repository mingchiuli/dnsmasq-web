use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use crate::config::model::{DnsRecords, ValidationIssue, ValidationLevel};

pub fn validate_records(records: &DnsRecords) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    validate_address(records, &mut issues);
    validate_host_record(records, &mut issues);
    validate_cname(records, &mut issues);
    validate_server(records, &mut issues);
    issues
}

pub fn has_errors(issues: &[ValidationIssue]) -> bool {
    issues
        .iter()
        .any(|issue| matches!(issue.level, ValidationLevel::Error))
}

fn validate_address(records: &DnsRecords, issues: &mut Vec<ValidationIssue>) {
    let mut seen = HashMap::<String, usize>::new();
    for (idx, record) in records.address.iter().enumerate() {
        let field = format!("address[{idx}]");
        if !is_domain_like(&record.domain) {
            error(issues, &field, "address domain is invalid");
        }
        if record.ip.parse::<IpAddr>().is_err() {
            error(issues, &field, "address ip is invalid");
        }

        let domain = record.domain.trim().to_ascii_lowercase();
        if let Some(first_idx) = seen.insert(domain, idx) {
            error(
                issues,
                &field,
                format!(
                    "duplicate address domain: {} also exists at address[{first_idx}]",
                    record.domain.trim()
                ),
            );
        }
    }
}

fn validate_host_record(records: &DnsRecords, issues: &mut Vec<ValidationIssue>) {
    for (idx, record) in records.host_record.iter().enumerate() {
        let field = format!("host_record[{idx}]");
        if record.names.is_empty() {
            error(issues, &field, "host-record requires at least one name");
        }
        if record.ips.is_empty() {
            error(issues, &field, "host-record requires at least one ip");
        }
        for name in &record.names {
            if !is_domain_like(name) {
                error(
                    issues,
                    &field,
                    format!("host-record name is invalid: {name}"),
                );
            }
        }
        for ip in &record.ips {
            if ip.parse::<IpAddr>().is_err() {
                error(issues, &field, format!("host-record ip is invalid: {ip}"));
            }
        }
    }
}

fn validate_cname(records: &DnsRecords, issues: &mut Vec<ValidationIssue>) {
    let mut aliases = HashSet::<String>::new();
    for (idx, record) in records.cname.iter().enumerate() {
        let field = format!("cname[{idx}]");
        if !is_domain_like(&record.alias) {
            error(issues, &field, "cname alias is invalid");
        }
        if !is_domain_like(&record.canonical) {
            error(issues, &field, "cname canonical target is invalid");
        }
        if record.alias.eq_ignore_ascii_case(&record.canonical) {
            error(
                issues,
                &field,
                "cname alias and canonical target cannot match",
            );
        }
        if !aliases.insert(record.alias.to_ascii_lowercase()) {
            warning(
                issues,
                &field,
                format!("duplicate cname alias: {}", record.alias),
            );
        }
    }
}

fn validate_server(records: &DnsRecords, issues: &mut Vec<ValidationIssue>) {
    let mut seen = HashSet::<String>::new();
    for (idx, record) in records.server.iter().enumerate() {
        let field = format!("server[{idx}]");
        if let Some(domain) = &record.domain
            && !domain.is_empty()
            && !is_domain_like(domain)
        {
            error(issues, &field, "server domain scope is invalid");
        }
        if !is_upstream_like(&record.upstream) {
            error(issues, &field, "server upstream is invalid");
        }
        let key = format!(
            "{}|{}",
            record
                .domain
                .as_deref()
                .unwrap_or_default()
                .to_ascii_lowercase(),
            record.upstream.to_ascii_lowercase()
        );
        if !seen.insert(key) {
            warning(
                issues,
                &field,
                format!("duplicate server upstream: {}", record.upstream),
            );
        }
    }
}

fn is_domain_like(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.len() > 253 || value.contains('/') || value.contains(',') {
        return false;
    }

    let value = value.strip_prefix('.').unwrap_or(value);
    if value.is_empty() {
        return false;
    }

    value.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    })
}

fn is_upstream_like(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.contains('/') || value.contains(',') {
        return false;
    }

    if value.parse::<IpAddr>().is_ok() {
        return true;
    }

    if let Some((host, port)) = value.rsplit_once('#') {
        return host.parse::<IpAddr>().is_ok() && port.parse::<u16>().is_ok();
    }

    is_domain_like(value)
}

fn error(issues: &mut Vec<ValidationIssue>, field: &str, message: impl Into<String>) {
    issues.push(ValidationIssue {
        level: ValidationLevel::Error,
        field: Some(field.into()),
        message: message.into(),
    });
}

fn warning(issues: &mut Vec<ValidationIssue>, field: &str, message: impl Into<String>) {
    issues.push(ValidationIssue {
        level: ValidationLevel::Warning,
        field: Some(field.into()),
        message: message.into(),
    });
}
