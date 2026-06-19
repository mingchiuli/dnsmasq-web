use crate::config::model::{
    ConfigLine, DnsRecords, MANAGED_BEGIN, MANAGED_END, ManagedRecord, ParsedConfig,
};
use crate::config::render::render_records_block;
use crate::error::{AppError, AppResult};

pub fn collect_records_from_config(config: &ParsedConfig) -> DnsRecords {
    if config.has_managed_block {
        collect_records_from_existing_block(config)
    } else {
        collect_records(&config.lines)
    }
}

pub fn collect_records(lines: &[ConfigLine]) -> DnsRecords {
    let mut records = DnsRecords::default();
    for line in lines {
        match line {
            ConfigLine::Managed(ManagedRecord::Address(record)) => {
                records.address.push(record.clone());
            }
            ConfigLine::Managed(ManagedRecord::HostRecord(record)) => {
                records.host_record.push(record.clone());
            }
            ConfigLine::Managed(ManagedRecord::Cname(record)) => {
                records.cname.push(record.clone());
            }
            ConfigLine::Managed(ManagedRecord::Server(record)) => {
                records.server.push(record.clone());
            }
            ConfigLine::Blank(_) | ConfigLine::Comment(_) | ConfigLine::RawDirective(_) => {}
        }
    }
    records
}

pub fn replace_managed_records(
    config: &ParsedConfig,
    records: DnsRecords,
) -> AppResult<ParsedConfig> {
    if config.has_managed_block {
        return replace_existing_block(config, records);
    }

    let mut lines = Vec::new();
    let mut inserted = false;

    for line in &config.lines {
        if matches!(line, ConfigLine::Managed(_)) {
            if !inserted {
                lines.extend(render_records_block(&records));
                inserted = true;
            }
            continue;
        }
        lines.push(line.clone());
    }

    if !inserted {
        if !lines.is_empty() {
            lines.push(ConfigLine::Blank(String::new()));
        }
        lines.extend(render_records_block(&records));
    }

    Ok(ParsedConfig {
        lines,
        has_managed_block: true,
    })
}

fn collect_records_from_existing_block(config: &ParsedConfig) -> DnsRecords {
    let mut lines = Vec::new();
    let mut in_block = false;

    for line in &config.lines {
        match line {
            ConfigLine::Comment(value) if value.trim() == MANAGED_BEGIN => {
                in_block = true;
            }
            ConfigLine::Comment(value) if value.trim() == MANAGED_END => {
                in_block = false;
            }
            _ if in_block => lines.push(line.clone()),
            _ => {}
        }
    }

    collect_records(&lines)
}

fn replace_existing_block(config: &ParsedConfig, records: DnsRecords) -> AppResult<ParsedConfig> {
    let mut lines = Vec::new();
    let mut in_block = false;
    let mut replaced = false;

    for line in &config.lines {
        match line {
            ConfigLine::Comment(value) if value.trim() == MANAGED_BEGIN => {
                if !replaced {
                    lines.extend(render_records_block(&records));
                    replaced = true;
                }
                in_block = true;
            }
            ConfigLine::Comment(value) if value.trim() == MANAGED_END => {
                in_block = false;
            }
            _ if in_block => {}
            _ => lines.push(line.clone()),
        }
    }

    if in_block {
        return Err(AppError::InvalidConfig(String::from(
            "managed records block is missing end marker",
        )));
    }

    if !replaced {
        lines.extend(render_records_block(&records));
    }

    Ok(ParsedConfig {
        lines,
        has_managed_block: true,
    })
}
