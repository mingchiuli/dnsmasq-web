use serde::{Deserialize, Serialize};

pub const MANAGED_BEGIN: &str = "# dnsmasqweb managed records begin";
pub const MANAGED_END: &str = "# dnsmasqweb managed records end";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DnsRecords {
    #[serde(default)]
    pub address: Vec<AddressRecord>,
    #[serde(default)]
    pub host_record: Vec<HostRecord>,
    #[serde(default)]
    pub cname: Vec<CnameRecord>,
    #[serde(default)]
    pub server: Vec<ServerRecord>,
}

impl DnsRecords {
    pub fn is_empty(&self) -> bool {
        self.address.is_empty()
            && self.host_record.is_empty()
            && self.cname.is_empty()
            && self.server.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressRecord {
    pub domain: String,
    pub ip: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostRecord {
    pub names: Vec<String>,
    pub ips: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CnameRecord {
    pub alias: String,
    pub canonical: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerRecord {
    pub domain: Option<String>,
    pub upstream: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ManagedRecord {
    Address(AddressRecord),
    HostRecord(HostRecord),
    Cname(CnameRecord),
    Server(ServerRecord),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigLine {
    Blank(String),
    Comment(String),
    Managed(ManagedRecord),
    RawDirective(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedConfig {
    pub lines: Vec<ConfigLine>,
    pub has_managed_block: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationLevel {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub level: ValidationLevel,
    pub field: Option<String>,
    pub message: String,
}
