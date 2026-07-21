use dnsmasqweb::config::model::{AddressRecord, DnsRecords, MANAGED_BEGIN, MANAGED_END};
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
    let error = parse_config(&input).expect_err("reject block");

    assert!(error.to_string().contains("missing end marker"));
}

#[test]
fn replace_records_preserves_opaque_lines_inside_managed_block() {
    let input = format!(
        "before=true\n{MANAGED_BEGIN}\n# keep this comment\n\ninterface=wg0\naddress=/old.example.internal/10.10.0.1\n# keep this tail\n{MANAGED_END}\nafter=true\n"
    );
    let parsed = parse_config(&input).expect("parse config");
    let records = DnsRecords {
        address: vec![AddressRecord {
            domain: String::from("new.example.internal"),
            ip: String::from("10.10.0.2"),
        }],
        ..DnsRecords::default()
    };

    let next = replace_managed_records(&parsed, records).expect("replace records");
    let rendered = render_config(&next);

    assert!(rendered.contains("# keep this comment\n\ninterface=wg0"));
    assert!(rendered.contains("# keep this tail"));
    assert!(rendered.contains("before=true"));
    assert!(rendered.contains("after=true"));
    assert!(rendered.contains("address=/new.example.internal/10.10.0.2"));
    assert!(!rendered.contains("old.example.internal"));
}

#[test]
fn replace_records_preserves_managed_directives_outside_the_block() {
    let input = format!(
        "address=/outside.example.internal/10.10.0.1\n{MANAGED_BEGIN}\naddress=/inside.example.internal/10.10.0.2\n{MANAGED_END}\n"
    );
    let parsed = parse_config(&input).expect("parse config");
    let records = DnsRecords {
        address: vec![AddressRecord {
            domain: String::from("replacement.example.internal"),
            ip: String::from("10.10.0.3"),
        }],
        ..DnsRecords::default()
    };

    let rendered = render_config(
        &replace_managed_records(&parsed, records).expect("replace managed block records"),
    );

    assert!(rendered.contains("address=/outside.example.internal/10.10.0.1"));
    assert!(rendered.contains("address=/replacement.example.internal/10.10.0.3"));
    assert!(!rendered.contains("inside.example.internal"));
}

#[test]
fn parser_rejects_invalid_managed_block_structure() {
    let unexpected_end = parse_config(&format!("{MANAGED_END}\n")).expect_err("unexpected end");
    assert!(unexpected_end.to_string().contains("unexpected end marker"));

    let nested = parse_config(&format!(
        "{MANAGED_BEGIN}\n{MANAGED_BEGIN}\n{MANAGED_END}\n"
    ))
    .expect_err("nested block");
    assert!(nested.to_string().contains("cannot be nested"));

    let duplicate = parse_config(&format!(
        "{MANAGED_BEGIN}\n{MANAGED_END}\n{MANAGED_BEGIN}\n{MANAGED_END}\n"
    ))
    .expect_err("duplicate blocks");
    assert!(
        duplicate
            .to_string()
            .contains("multiple managed records blocks")
    );
}
