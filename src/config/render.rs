use crate::config::model::{
    AddressRecord, CnameRecord, ConfigLine, DnsRecords, HostRecord, MANAGED_BEGIN, MANAGED_END,
    ManagedRecord, ParsedConfig, ServerRecord,
};

pub fn render_config(config: &ParsedConfig) -> String {
    let mut output = String::new();
    for (idx, line) in config.lines.iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }
        output.push_str(&render_line(line));
    }
    output.push('\n');
    output
}

pub fn render_records_block(records: &DnsRecords) -> Vec<ConfigLine> {
    let mut lines = Vec::new();
    lines.push(ConfigLine::Comment(MANAGED_BEGIN.into()));

    for record in &records.address {
        lines.push(ConfigLine::Managed(ManagedRecord::Address(record.clone())));
    }
    for record in &records.host_record {
        lines.push(ConfigLine::Managed(ManagedRecord::HostRecord(
            record.clone(),
        )));
    }
    for record in &records.cname {
        lines.push(ConfigLine::Managed(ManagedRecord::Cname(record.clone())));
    }
    for record in &records.server {
        lines.push(ConfigLine::Managed(ManagedRecord::Server(record.clone())));
    }

    lines.push(ConfigLine::Comment(MANAGED_END.into()));
    lines
}

pub fn render_line(line: &ConfigLine) -> String {
    match line {
        ConfigLine::Blank(value) | ConfigLine::Comment(value) | ConfigLine::RawDirective(value) => {
            value.clone()
        }
        ConfigLine::Managed(record) => render_managed_record(record),
    }
}

pub fn render_managed_record(record: &ManagedRecord) -> String {
    match record {
        ManagedRecord::Address(record) => render_address(record),
        ManagedRecord::HostRecord(record) => render_host_record(record),
        ManagedRecord::Cname(record) => render_cname(record),
        ManagedRecord::Server(record) => render_server(record),
    }
}

pub fn render_address(record: &AddressRecord) -> String {
    format!("address=/{}/{}", record.domain.trim(), record.ip.trim())
}

pub fn render_host_record(record: &HostRecord) -> String {
    let values = record
        .names
        .iter()
        .chain(record.ips.iter())
        .map(String::as_str)
        .collect::<Vec<_>>();
    format!("host-record={}", values.join(","))
}

pub fn render_cname(record: &CnameRecord) -> String {
    format!("cname={},{}", record.alias.trim(), record.canonical.trim())
}

pub fn render_server(record: &ServerRecord) -> String {
    match record
        .domain
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(domain) => format!("server=/{}/{}", domain, record.upstream.trim()),
        None => format!("server={}", record.upstream.trim()),
    }
}
