use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    ZhCn,
    En,
}

impl Locale {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::ZhCn => Self::En,
            Self::En => Self::ZhCn,
        }
    }
}

impl From<&str> for Locale {
    fn from(value: &str) -> Self {
        let primary = value
            .split(',')
            .next()
            .unwrap_or_default()
            .split(';')
            .next()
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();

        if primary == "en" || primary.starts_with("en-") {
            Self::En
        } else {
            Self::ZhCn
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
    Active,
    Actions,
    Add,
    Address,
    AddressDomainPlaceholder,
    AddressEmpty,
    AddressHelp,
    AddressIpPlaceholder,
    Apply,
    Alias,
    BackupId,
    BackupDeleteConfirm,
    BackupDeleted,
    BackupSize,
    Backups,
    Cancel,
    ChangePassword,
    Cname,
    CnameAliasPlaceholder,
    CnameCanonicalPlaceholder,
    CnameEmpty,
    ConfigChanged,
    ConfigRefreshed,
    Confirm,
    ConfirmPassword,
    CurrentPassword,
    Delete,
    Domain,
    DomainScope,
    Edit,
    HostRecord,
    HostRecordEmpty,
    HostRecordIpsPlaceholder,
    HostRecordNamesPlaceholder,
    Inactive,
    Ip,
    LocaleSwitch,
    Login,
    LoginRequired,
    Logout,
    Name,
    NewPassword,
    Password,
    PasswordChanged,
    PasswordsDoNotMatch,
    RawConfig,
    RawConfigSaved,
    RawConfigSavedApplied,
    RecordsSaved,
    RecordsSavedApplied,
    RecordType,
    ResolutionMode,
    LocalOnly,
    ForwardUpstream,
    ServerLocalHelp,
    ServerLocalDomainPlaceholder,
    ServerLocalInvalid,
    ServerForwardInvalid,
    Refresh,
    Restore,
    RestoreApplied,
    Save,
    Server,
    ServerDomainPlaceholder,
    ServerEmpty,
    ServerUpstreamPlaceholder,
    SetPassword,
    SetupPassword,
    TestConfig,
    TestPassed,
    Upstream,
    UnmanagedLines,
}

pub const fn t(locale: Locale, msg: Msg) -> &'static str {
    match locale {
        Locale::ZhCn => zh_cn(msg),
        Locale::En => en(msg),
    }
}

const fn zh_cn(msg: Msg) -> &'static str {
    match msg {
        Msg::Active => "active",
        Msg::Actions => "操作",
        Msg::Add => "新增",
        Msg::Address => "Address",
        Msg::AddressDomainPlaceholder => "gateway.example.com 或 .example.com",
        Msg::AddressEmpty => "暂无 address 记录",
        Msg::AddressHelp => {
            "A 对应 IPv4，AAAA 对应 IPv6。同一域名可各配置一条。仅配置地址不一定阻止其他类型的查询转发到上游；需要时请在 Server 中添加该域名的“仅本地解析”规则。"
        }
        Msg::AddressIpPlaceholder => "10.10.0.1 或 fd00::1",
        Msg::Apply => "一键生效",
        Msg::Alias => "别名",
        Msg::BackupId => "备份 ID",
        Msg::BackupDeleteConfirm => "确认删除这个备份？",
        Msg::BackupDeleted => "备份已删除",
        Msg::BackupSize => "bytes",
        Msg::Backups => "备份",
        Msg::Cancel => "取消",
        Msg::ChangePassword => "修改密码",
        Msg::Cname => "CNAME",
        Msg::CnameAliasPlaceholder => "alias.example.com",
        Msg::CnameCanonicalPlaceholder => "target.example.com",
        Msg::CnameEmpty => "暂无 cname 记录",
        Msg::ConfigChanged => "配置已被其他操作修改，请刷新后重试",
        Msg::ConfigRefreshed => "配置已刷新",
        Msg::Confirm => "确认",
        Msg::ConfirmPassword => "确认密码",
        Msg::CurrentPassword => "当前密码",
        Msg::Delete => "删除",
        Msg::Domain => "域名",
        Msg::DomainScope => "域名范围",
        Msg::Edit => "编辑",
        Msg::HostRecord => "Host Record",
        Msg::HostRecordEmpty => "暂无 host-record 记录",
        Msg::HostRecordIpsPlaceholder => "10.10.0.2, fd00::2",
        Msg::HostRecordNamesPlaceholder => "host.example.com, alias.example.com",
        Msg::Inactive => "inactive",
        Msg::Ip => "IP",
        Msg::LocaleSwitch => "English",
        Msg::Login => "登录",
        Msg::LoginRequired => "请重新登录",
        Msg::Logout => "退出",
        Msg::Name => "名称",
        Msg::NewPassword => "新密码",
        Msg::Password => "密码",
        Msg::PasswordChanged => "密码已修改",
        Msg::PasswordsDoNotMatch => "两次输入的密码不一致",
        Msg::RawConfig => "原始文本",
        Msg::RawConfigSaved => "原始配置已保存，尚未应用；点击“一键生效”加载配置",
        Msg::RawConfigSavedApplied => "原始配置已保存并生效",
        Msg::RecordsSaved => "配置已保存，尚未应用；点击“一键生效”加载配置",
        Msg::RecordsSavedApplied => "配置已保存并生效",
        Msg::RecordType => "记录类型",
        Msg::ResolutionMode => "解析模式",
        Msg::LocalOnly => "仅本地解析",
        Msg::ForwardUpstream => "转发到上游",
        Msg::ServerLocalHelp => {
            "仅本地解析适用于指定域名及其子域名：使用本地记录回答，不向上游查询。更具体的域名转发规则仍可覆盖此规则。原始文本中的 local= 与此等价，仍需在原始文本中维护。"
        }
        Msg::ServerLocalDomainPlaceholder => "必填，例如 app.example.com",
        Msg::ServerLocalInvalid => "仅本地解析需要填写有效域名。",
        Msg::ServerForwardInvalid => "请填写有效上游和域名；域名可留空，表示默认上游。",
        Msg::Refresh => "刷新",
        Msg::Restore => "恢复",
        Msg::RestoreApplied => "备份已恢复并生效",
        Msg::Save => "保存",
        Msg::Server => "Server",
        Msg::ServerDomainPlaceholder => "留空表示默认上游",
        Msg::ServerEmpty => "暂无 server 记录",
        Msg::ServerUpstreamPlaceholder => "223.5.5.5 或 10.10.0.1#5353",
        Msg::SetPassword => "设置密码",
        Msg::SetupPassword => "设置管理密码",
        Msg::TestConfig => "测试配置",
        Msg::TestPassed => "测试通过",
        Msg::Upstream => "上游",
        Msg::UnmanagedLines => "未受管行",
    }
}

const fn en(msg: Msg) -> &'static str {
    match msg {
        Msg::Active => "active",
        Msg::Actions => "Actions",
        Msg::Add => "Add",
        Msg::Address => "Address",
        Msg::AddressDomainPlaceholder => "gateway.example.com or .example.com",
        Msg::AddressEmpty => "No address records",
        Msg::AddressHelp => {
            "A is IPv4; AAAA is IPv6. Each domain can have one of each. Address rules alone may still forward other query types upstream. To prevent this, add a local-only rule for the domain in Server."
        }
        Msg::AddressIpPlaceholder => "10.10.0.1 or fd00::1",
        Msg::Apply => "Apply",
        Msg::Alias => "Alias",
        Msg::BackupId => "Backup ID",
        Msg::BackupDeleteConfirm => "Delete this backup?",
        Msg::BackupDeleted => "Backup deleted",
        Msg::BackupSize => "bytes",
        Msg::Backups => "Backups",
        Msg::Cancel => "Cancel",
        Msg::ChangePassword => "Change Password",
        Msg::Cname => "CNAME",
        Msg::CnameAliasPlaceholder => "alias.example.com",
        Msg::CnameCanonicalPlaceholder => "target.example.com",
        Msg::CnameEmpty => "No cname records",
        Msg::ConfigChanged => "Configuration changed; refresh and try again",
        Msg::ConfigRefreshed => "Configuration refreshed",
        Msg::Confirm => "Confirm",
        Msg::ConfirmPassword => "Confirm Password",
        Msg::CurrentPassword => "Current Password",
        Msg::Delete => "Delete",
        Msg::Domain => "Domain",
        Msg::DomainScope => "Domain Scope",
        Msg::Edit => "Edit",
        Msg::HostRecord => "Host Record",
        Msg::HostRecordEmpty => "No host-record records",
        Msg::HostRecordIpsPlaceholder => "10.10.0.2, fd00::2",
        Msg::HostRecordNamesPlaceholder => "host.example.com, alias.example.com",
        Msg::Inactive => "inactive",
        Msg::Ip => "IP",
        Msg::LocaleSwitch => "简体中文",
        Msg::Login => "Log in",
        Msg::LoginRequired => "Please log in again",
        Msg::Logout => "Log out",
        Msg::Name => "Name",
        Msg::NewPassword => "New Password",
        Msg::Password => "Password",
        Msg::PasswordChanged => "Password changed",
        Msg::PasswordsDoNotMatch => "Passwords do not match",
        Msg::RawConfig => "Raw Config",
        Msg::RawConfigSaved => "Raw config saved, not yet applied; click Apply to load it",
        Msg::RawConfigSavedApplied => "Raw config saved and applied",
        Msg::RecordsSaved => "Configuration saved, not yet applied; click Apply to load it",
        Msg::RecordsSavedApplied => "Configuration saved and applied",
        Msg::RecordType => "Record type",
        Msg::ResolutionMode => "Resolution mode",
        Msg::LocalOnly => "Local only",
        Msg::ForwardUpstream => "Forward upstream",
        Msg::ServerLocalHelp => {
            "Local-only rules answer from local records without querying upstream for the domain and its subdomains. More specific forwarding rules can override them. Equivalent local= directives in Raw Config must still be maintained there."
        }
        Msg::ServerLocalDomainPlaceholder => "Required, e.g. app.example.com",
        Msg::ServerLocalInvalid => "Local-only resolution requires a valid domain.",
        Msg::ServerForwardInvalid => {
            "Enter a valid upstream and domain; leave the domain empty for a default upstream."
        }
        Msg::Refresh => "Refresh",
        Msg::Restore => "Restore",
        Msg::RestoreApplied => "Backup restored and applied",
        Msg::Save => "Save",
        Msg::Server => "Server",
        Msg::ServerDomainPlaceholder => "Leave empty for default upstream",
        Msg::ServerEmpty => "No server records",
        Msg::ServerUpstreamPlaceholder => "223.5.5.5 or 10.10.0.1#5353",
        Msg::SetPassword => "Set Password",
        Msg::SetupPassword => "Set Admin Password",
        Msg::TestConfig => "Test Config",
        Msg::TestPassed => "Test passed",
        Msg::Upstream => "Upstream",
        Msg::UnmanagedLines => "Unmanaged lines",
    }
}
