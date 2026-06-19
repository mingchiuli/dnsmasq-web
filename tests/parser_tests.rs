use dnsmasqweb::config::model::{AddressRecord, DnsRecords, MANAGED_BEGIN};
use dnsmasqweb::config::parser::parse_config;
use dnsmasqweb::config::records::{
    collect_records, collect_records_from_config, replace_managed_records,
};
use dnsmasqweb::config::render::render_config;
use dnsmasqweb::config::validate::{has_errors, validate_records};

const SAMPLE: &str = include_str!("fixtures/dnsmasq.conf");

#[test]
fn parses_current_dnsmasq_shape() {
    let parsed = parse_config(SAMPLE).expect("parse sample");
    let records = collect_records(&parsed.lines);

    assert_eq!(records.address.len(), 14);
    assert_eq!(records.server.len(), 2);
    assert_eq!(records.server[0].domain, None);
    assert_eq!(records.server[0].upstream, "223.5.5.5");
    assert!(records.host_record.is_empty());
    assert!(records.cname.is_empty());
}

#[test]
fn validation_rejects_duplicate_address_domain() {
    let records = DnsRecords {
        address: vec![
            AddressRecord {
                domain: String::from("app.example.internal"),
                ip: String::from("10.10.0.1"),
            },
            AddressRecord {
                domain: String::from("app.example.internal"),
                ip: String::from("10.10.0.2"),
            },
        ],
        ..DnsRecords::default()
    };
    let issues = validate_records(&records);

    assert!(has_errors(&issues));
    assert!(
        issues
            .iter()
            .any(|issue| issue.message.contains("duplicate address domain")
                && issue.message.contains("app.example.internal")
                && issue.field.as_deref() == Some("address[1]"))
    );
}

#[test]
fn replace_records_preserves_unmanaged_lines() {
    let parsed = parse_config(SAMPLE).expect("parse sample");
    let mut records = collect_records(&parsed.lines);
    records
        .address
        .retain(|record| record.domain != "mg-test.example.internal");

    let next = replace_managed_records(&parsed, records).expect("replace records");
    let rendered = render_config(&next);

    assert!(rendered.contains("interface=wg0"));
    assert!(rendered.contains("bind-interfaces"));
    assert!(rendered.contains("no-hosts"));
    assert!(rendered.contains("#log-queries"));
    assert!(!rendered.contains("mg-test.example.internal"));
    assert!(rendered.contains("# dnsmasqweb managed records begin"));
}

#[test]
fn existing_block_collects_only_records_inside_block() {
    let input = format!(
        "address=/outside.example.internal/10.10.0.1\n{MANAGED_BEGIN}\naddress=/inside.example.internal/10.10.0.2\n# dnsmasqweb managed records end\n"
    );
    let parsed = parse_config(&input).expect("parse config");
    let records = collect_records_from_config(&parsed);

    assert_eq!(records.address.len(), 1);
    assert_eq!(records.address[0].domain, "inside.example.internal");
}

#[test]
fn replace_records_rejects_unclosed_managed_block() {
    let input = format!("{MANAGED_BEGIN}\naddress=/inside.example.internal/10.10.0.2\nno-hosts\n");
    let parsed = parse_config(&input).expect("parse config");
    let error = replace_managed_records(&parsed, DnsRecords::default()).expect_err("reject block");

    assert!(error.to_string().contains("missing end marker"));
}
