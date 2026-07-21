use crate::config::model::{ConfigLine, DnsRecords, ManagedBlock, ManagedRecord, ParsedConfig};
use crate::config::render::{render_managed_records, render_records_block};
use crate::error::{AppError, AppResult};

pub fn collect_records_from_config(config: &ParsedConfig) -> DnsRecords {
    match config.managed_block {
        Some(block) => config
            .lines
            .get(block.begin + 1..block.end)
            .map(collect_records)
            .unwrap_or_default(),
        None => collect_records(&config.lines),
    }
}

pub fn managed_record_count(config: &ParsedConfig) -> usize {
    let lines = match config.managed_block {
        Some(block) => config.lines.get(block.begin + 1..block.end),
        None => Some(config.lines.as_slice()),
    };
    lines
        .map(|lines| {
            lines
                .iter()
                .filter(|line| matches!(line, ConfigLine::Managed(_)))
                .count()
        })
        .unwrap_or_default()
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
            ConfigLine::Blank(_)
            | ConfigLine::Comment(_)
            | ConfigLine::ManagedBlockBegin(_)
            | ConfigLine::ManagedBlockEnd(_)
            | ConfigLine::RawDirective(_) => {}
        }
    }
    records
}

pub fn replace_managed_records(
    config: &ParsedConfig,
    records: DnsRecords,
) -> AppResult<ParsedConfig> {
    if let Some(block) = config.managed_block {
        return replace_existing_block(config, block, records);
    }

    let mut lines = Vec::new();
    let mut inserted = false;
    let mut records = Some(records);
    let mut managed_block = None;

    for line in &config.lines {
        if matches!(line, ConfigLine::Managed(_)) {
            if !inserted {
                if let Some(records) = records.take() {
                    managed_block = Some(append_managed_block(&mut lines, records));
                }
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
        if let Some(records) = records {
            managed_block = Some(append_managed_block(&mut lines, records));
        }
    }

    Ok(ParsedConfig {
        lines,
        managed_block,
    })
}

fn append_managed_block(lines: &mut Vec<ConfigLine>, records: DnsRecords) -> ManagedBlock {
    let begin = lines.len();
    lines.extend(render_records_block(records));
    ManagedBlock {
        begin,
        end: lines.len() - 1,
    }
}

fn replace_existing_block(
    config: &ParsedConfig,
    block: ManagedBlock,
    records: DnsRecords,
) -> AppResult<ParsedConfig> {
    let Some(ConfigLine::ManagedBlockBegin(begin_marker)) = config.lines.get(block.begin) else {
        return Err(AppError::InvalidConfig(String::from(
            "managed records block has an invalid begin marker",
        )));
    };
    let Some(ConfigLine::ManagedBlockEnd(end_marker)) = config.lines.get(block.end) else {
        return Err(AppError::InvalidConfig(String::from(
            "managed records block has an invalid end marker",
        )));
    };
    let Some(contents) = config.lines.get(block.begin + 1..block.end) else {
        return Err(AppError::InvalidConfig(String::from(
            "managed records block range is invalid",
        )));
    };

    let mut lines = Vec::new();
    lines.extend_from_slice(&config.lines[..block.begin]);
    let begin = lines.len();
    lines.push(ConfigLine::ManagedBlockBegin(begin_marker.clone()));

    let replacement = render_managed_records(records);
    let first_managed = contents
        .iter()
        .position(|line| matches!(line, ConfigLine::Managed(_)));
    if first_managed.is_none() {
        lines.extend(replacement.iter().cloned());
    }
    for (index, line) in contents.iter().enumerate() {
        if first_managed == Some(index) {
            lines.extend(replacement.iter().cloned());
        }
        if !matches!(line, ConfigLine::Managed(_)) {
            lines.push(line.clone());
        }
    }
    let end = lines.len();
    lines.push(ConfigLine::ManagedBlockEnd(end_marker.clone()));
    lines.extend_from_slice(&config.lines[block.end + 1..]);

    Ok(ParsedConfig {
        lines,
        managed_block: Some(ManagedBlock { begin, end }),
    })
}
