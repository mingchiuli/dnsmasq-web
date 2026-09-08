use leptos::prelude::*;

use crate::config::model::{AddressRecord, DnsRecords, ServerRecord};
use crate::config::validate::{has_errors, validate_records};

use super::{EditableRow, upsert_row};

fn domain_key(domain: &str) -> String {
    let domain = domain.trim();
    domain
        .strip_prefix('.')
        .unwrap_or(domain)
        .to_ascii_lowercase()
}

fn matches_local(record: &ServerRecord, domain: &str) -> bool {
    !domain.trim().is_empty()
        && record.upstream.trim().is_empty()
        && record
            .domain
            .as_deref()
            .is_some_and(|scope| domain_key(scope) == domain_key(domain))
}

pub(super) fn has_local_rule(rows: &[EditableRow<ServerRecord>], domain: &str) -> bool {
    rows.iter()
        .any(|row| row.value.with(|record| matches_local(record, domain)))
}

fn set_local_rule(rows: RwSignal<Vec<EditableRow<ServerRecord>>>, domain: &str, enabled: bool) {
    rows.update(|rows| {
        let mut retained = false;
        rows.retain(|row| {
            if !row
                .value
                .with_untracked(|record| matches_local(record, domain))
            {
                return true;
            }
            if enabled && !retained {
                retained = true;
                true
            } else {
                false
            }
        });
        if enabled && !retained {
            rows.push(EditableRow::new(ServerRecord {
                domain: Some(domain.trim().into()),
                upstream: String::new(),
            }));
        }
    });
}

/// Validate the proposed Address list before changing either draft. Rules for
/// the old domain are deliberately retained when an Address is renamed.
pub(super) fn save_address(
    rows: RwSignal<Vec<EditableRow<AddressRecord>>>,
    servers: RwSignal<Vec<EditableRow<ServerRecord>>>,
    id: Option<u64>,
    record: AddressRecord,
    local_only: bool,
) -> bool {
    let mut proposed = rows.with_untracked(|rows| {
        rows.iter()
            .filter(|row| Some(row.id) != id)
            .map(|row| row.value.get_untracked())
            .collect::<Vec<_>>()
    });
    proposed.push(record.clone());
    if has_errors(&validate_records(&DnsRecords {
        address: proposed,
        ..DnsRecords::default()
    })) {
        return false;
    }

    let domain = record.domain.clone();
    upsert_row(rows, id, record);
    set_local_rule(servers, &domain, local_only);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::{MANAGED_BEGIN, MANAGED_END};
    use crate::config::parser::parse_config;
    use crate::config::records::{collect_records_from_config, replace_managed_records};
    use crate::config::render::render_config;
    use crate::ui::tables::{editable_rows, remove_row, row_values};

    fn address(domain: &str, ip: &str) -> AddressRecord {
        AddressRecord {
            domain: domain.into(),
            ip: ip.into(),
        }
    }

    fn server(domain: Option<&str>, upstream: &str) -> ServerRecord {
        ServerRecord {
            domain: domain.map(String::from),
            upstream: upstream.into(),
        }
    }

    #[test]
    fn dual_stack_shares_existing_rules_and_unchecking_preserves_forwarding() {
        Owner::new().with(|| {
            let addresses = RwSignal::new(Vec::new());
            let servers = RwSignal::new(editable_rows(vec![
                server(Some(" .App.Example "), ""),
                server(Some("app.example"), " "),
                server(Some("app.example"), "10.0.0.53"),
                server(None, "1.1.1.1"),
            ]));
            let original_id = servers.with(|rows| rows[0].id);
            assert!(servers.with(|rows| has_local_rule(rows, "APP.EXAMPLE")));
            assert!(!servers.with(|rows| has_local_rule(rows, "child.app.example")));
            assert!(save_address(
                addresses,
                servers,
                None,
                address("app.example", "10.0.0.1"),
                true
            ));
            assert_eq!(servers.with(Vec::len), 3);
            assert_eq!(servers.with(|rows| rows[0].id), original_id);
            assert!(save_address(
                addresses,
                servers,
                None,
                address("app.example", "fd00::1"),
                true
            ));
            assert_eq!(servers.with(Vec::len), 3);
            let ipv6_id = addresses.with(|rows| rows[1].id);
            assert!(save_address(
                addresses,
                servers,
                Some(ipv6_id),
                address("app.example", "fd00::1"),
                false
            ));
            assert!(!servers.with(|rows| has_local_rule(rows, "app.example")));
            assert_eq!(
                servers.with(|rows| row_values(rows)),
                vec![
                    server(Some("app.example"), "10.0.0.53"),
                    server(None, "1.1.1.1"),
                ]
            );
            assert_eq!(addresses.with(Vec::len), 2);
        });
    }

    #[test]
    fn invalid_address_does_not_change_either_draft() {
        Owner::new().with(|| {
            let addresses = RwSignal::new(Vec::new());
            let servers = RwSignal::new(Vec::new());
            assert!(!save_address(
                addresses,
                servers,
                None,
                address("bad/domain", "10.0.0.1"),
                true
            ));
            assert!(!save_address(
                addresses,
                servers,
                None,
                address("app.example", "invalid"),
                true
            ));
            assert!(addresses.with(Vec::is_empty));
            assert!(servers.with(Vec::is_empty));
            assert!(save_address(
                addresses,
                servers,
                None,
                address("app.example", "10.0.0.1"),
                true
            ));
            assert!(!save_address(
                addresses,
                servers,
                None,
                address("app.example", "10.0.0.2"),
                false
            ));
            assert_eq!(
                addresses.with(|rows| row_values(rows)),
                vec![address("app.example", "10.0.0.1")]
            );
            assert!(servers.with(|rows| has_local_rule(rows, "app.example")));
        });
    }

    #[test]
    fn rename_and_delete_retain_old_domain_rules() {
        Owner::new().with(|| {
            let addresses = RwSignal::new(Vec::new());
            let servers = RwSignal::new(Vec::new());
            assert!(save_address(
                addresses,
                servers,
                None,
                address("old.example", "10.0.0.1"),
                true
            ));
            let id = addresses.with(|rows| rows[0].id);
            assert!(save_address(
                addresses,
                servers,
                Some(id),
                address("new.example", "10.0.0.1"),
                false
            ));
            assert!(servers.with(|rows| has_local_rule(rows, "old.example")));
            assert!(!servers.with(|rows| has_local_rule(rows, "new.example")));
            assert!(save_address(
                addresses,
                servers,
                Some(id),
                address("new.example", "10.0.0.1"),
                true
            ));
            remove_row(addresses, id);
            assert!(addresses.with(Vec::is_empty));
            assert!(servers.with(|rows| has_local_rule(rows, "old.example")));
            assert!(servers.with(|rows| has_local_rule(rows, "new.example")));
        });
    }

    #[test]
    fn saving_and_reloading_preserves_raw_rules_and_shares_server_changes() {
        Owner::new().with(|| {
            let input = format!("server=/outside.example/\n{MANAGED_BEGIN}\n# keep\n\nlocal=/app.example/\naddress=/app.example/10.0.0.1\n{MANAGED_END}\n");
            let parsed = parse_config(&input).expect("parse config");
            let records = collect_records_from_config(&parsed);
            let addresses = RwSignal::new(editable_rows(records.address));
            let servers = RwSignal::new(editable_rows(records.server));
            assert!(!servers.with(|rows| has_local_rule(rows, "app.example")));
            let id = addresses.with(|rows| rows[0].id);
            assert!(save_address(addresses, servers, Some(id), address("app.example", "10.0.0.1"), true));
            let rendered = render_config(&replace_managed_records(&parsed, DnsRecords {
                address: addresses.with(|rows| row_values(rows)),
                server: servers.with(|rows| row_values(rows)),
                ..DnsRecords::default()
            }).expect("save managed records"));
            assert!(rendered.contains("# keep\n\nlocal=/app.example/"));
            assert!(rendered.starts_with("server=/outside.example/\n"));
            let loaded = collect_records_from_config(&parse_config(&rendered).expect("reload"));
            servers.set(editable_rows(loaded.server));
            assert!(servers.with(|rows| has_local_rule(rows, "app.example")));
            let server_id = servers.with(|rows| rows[0].id);
            upsert_row(servers, Some(server_id), server(Some("app.example"), "10.0.0.53"));
            assert!(!servers.with(|rows| has_local_rule(rows, "app.example")));
            upsert_row(servers, Some(server_id), server(Some("APP.EXAMPLE"), ""));
            assert!(servers.with(|rows| has_local_rule(rows, "app.example")));
        });
    }
}
