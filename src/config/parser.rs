use crate::config::model::{
    AddressRecord, CnameRecord, ConfigLine, HostRecord, MANAGED_BEGIN, MANAGED_END, ManagedBlock,
    ManagedRecord, ParsedConfig, ServerRecord,
};
use crate::error::{AppError, AppResult};

pub fn parse_config(input: &str) -> AppResult<ParsedConfig> {
    let mut lines = Vec::new();
    let mut open_block = None::<(usize, usize)>;
    let mut managed_block = None;

    for (idx, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();

        if line == MANAGED_BEGIN {
            if open_block.is_some() {
                return Err(AppError::ParseLine {
                    line: idx + 1,
                    message: String::from("managed records block cannot be nested"),
                });
            }
            if managed_block.is_some() {
                return Err(AppError::ParseLine {
                    line: idx + 1,
                    message: String::from("multiple managed records blocks are not allowed"),
                });
            }
            open_block = Some((lines.len(), idx + 1));
            lines.push(ConfigLine::ManagedBlockBegin(raw_line.into()));
            continue;
        }

        if line == MANAGED_END {
            let Some((begin, _)) = open_block.take() else {
                return Err(AppError::ParseLine {
                    line: idx + 1,
                    message: String::from("managed records block has an unexpected end marker"),
                });
            };
            let end = lines.len();
            lines.push(ConfigLine::ManagedBlockEnd(raw_line.into()));
            managed_block = Some(ManagedBlock { begin, end });
            continue;
        }

        if line.is_empty() {
            lines.push(ConfigLine::Blank(raw_line.into()));
        } else if line.starts_with('#') {
            lines.push(ConfigLine::Comment(raw_line.into()));
        } else if let Some(record) =
            parse_managed_line(line).map_err(|message| AppError::ParseLine {
                line: idx + 1,
                message,
            })?
        {
            lines.push(ConfigLine::Managed(record));
        } else {
            lines.push(ConfigLine::RawDirective(raw_line.into()));
        }
    }

    if let Some((_, line)) = open_block {
        return Err(AppError::ParseLine {
            line,
            message: String::from("managed records block is missing end marker"),
        });
    }

    Ok(ParsedConfig {
        lines,
        managed_block,
    })
}

pub fn parse_records(input: &str) -> AppResult<crate::config::model::DnsRecords> {
    let parsed = parse_config(input)?;
    Ok(crate::config::records::collect_records_from_config(&parsed))
}

fn parse_managed_line(line: &str) -> Result<Option<ManagedRecord>, String> {
    if let Some(value) = line.strip_prefix("address=") {
        return parse_address(value).map(|record| Some(ManagedRecord::Address(record)));
    }

    if let Some(value) = line.strip_prefix("host-record=") {
        return parse_host_record(value).map(|record| Some(ManagedRecord::HostRecord(record)));
    }

    if let Some(value) = line.strip_prefix("cname=") {
        return parse_cname(value).map(|record| Some(ManagedRecord::Cname(record)));
    }

    if let Some(value) = line.strip_prefix("server=") {
        return parse_server(value).map(|record| Some(ManagedRecord::Server(record)));
    }

    Ok(None)
}

fn parse_address(value: &str) -> Result<AddressRecord, String> {
    let mut parts = value.split('/');
    let first = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    let ip = parts.next().unwrap_or_default();

    if !first.is_empty() || domain.is_empty() || ip.is_empty() || parts.next().is_some() {
        return Err(String::from("expected address=/domain/ip"));
    }

    Ok(AddressRecord {
        domain: domain.into(),
        ip: ip.into(),
    })
}

fn parse_host_record(value: &str) -> Result<HostRecord, String> {
    let items = split_csv(value);
    if items.len() < 2 {
        return Err(String::from("expected host-record=name,ip[,ip...]"));
    }

    let mut names = Vec::new();
    let mut ips = Vec::new();
    for item in items {
        if item.parse::<std::net::IpAddr>().is_ok() {
            ips.push(item);
        } else {
            names.push(item);
        }
    }

    if names.is_empty() || ips.is_empty() {
        return Err(String::from(
            "host-record requires at least one name and one ip",
        ));
    }

    Ok(HostRecord { names, ips })
}

fn parse_cname(value: &str) -> Result<CnameRecord, String> {
    match <[String; 2]>::try_from(split_csv(value)) {
        Ok([alias, canonical]) => Ok(CnameRecord { alias, canonical }),
        _ => Err(String::from("expected cname=alias,canonical")),
    }
}

fn parse_server(value: &str) -> Result<ServerRecord, String> {
    if let Some(rest) = value.strip_prefix('/') {
        let mut parts = rest.split('/');
        let domain = parts.next().unwrap_or_default();
        let upstream = parts.next().unwrap_or_default();
        if domain.is_empty() || upstream.is_empty() || parts.next().is_some() {
            return Err(String::from("expected server=/domain/upstream"));
        }

        Ok(ServerRecord {
            domain: Some(domain.into()),
            upstream: upstream.into(),
        })
    } else if value.trim().is_empty() {
        Err(String::from("server requires an upstream"))
    } else {
        Ok(ServerRecord {
            domain: None,
            upstream: value.into(),
        })
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
