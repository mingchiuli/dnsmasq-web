#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
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

impl Default for Locale {
    fn default() -> Self {
        Self::ZhCn
    }
}

impl From<&str> for Locale {
    fn from(value: &str) -> Self {
        match value {
            "en" => Self::En,
            _ => Self::ZhCn,
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
    Apply,
    Alias,
    BackupId,
    BackupSize,
    Backups,
    Cancel,
    Cname,
    CnameAliasPlaceholder,
    CnameCanonicalPlaceholder,
    CnameEmpty,
    ConfigRefreshed,
    Confirm,
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
    Loading,
    Login,
    LoginRequired,
    Logout,
    Name,
    Password,
    RawConfig,
    RawConfigSaved,
    RawConfigSavedApplied,
    RecordsSaved,
    RecordsSavedApplied,
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
        Msg::Apply => "一键生效",
        Msg::Alias => "别名",
        Msg::BackupId => "备份 ID",
        Msg::BackupSize => "bytes",
        Msg::Backups => "备份",
        Msg::Cancel => "取消",
        Msg::Cname => "CNAME",
        Msg::CnameAliasPlaceholder => "alias.example.com",
        Msg::CnameCanonicalPlaceholder => "target.example.com",
        Msg::CnameEmpty => "暂无 cname 记录",
        Msg::ConfigRefreshed => "配置已刷新",
        Msg::Confirm => "确认",
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
        Msg::Loading => "加载中",
        Msg::Login => "登录",
        Msg::LoginRequired => "请重新登录",
        Msg::Logout => "退出",
        Msg::Name => "名称",
        Msg::Password => "密码",
        Msg::RawConfig => "原始文本",
        Msg::RawConfigSaved => "原始配置已保存",
        Msg::RawConfigSavedApplied => "原始配置已保存并生效",
        Msg::RecordsSaved => "配置已保存",
        Msg::RecordsSavedApplied => "配置已保存并生效",
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
        Msg::Apply => "Apply",
        Msg::Alias => "Alias",
        Msg::BackupId => "Backup ID",
        Msg::BackupSize => "bytes",
        Msg::Backups => "Backups",
        Msg::Cancel => "Cancel",
        Msg::Cname => "CNAME",
        Msg::CnameAliasPlaceholder => "alias.example.com",
        Msg::CnameCanonicalPlaceholder => "target.example.com",
        Msg::CnameEmpty => "No cname records",
        Msg::ConfigRefreshed => "Configuration refreshed",
        Msg::Confirm => "Confirm",
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
        Msg::Loading => "Loading",
        Msg::Login => "Log in",
        Msg::LoginRequired => "Please log in again",
        Msg::Logout => "Log out",
        Msg::Name => "Name",
        Msg::Password => "Password",
        Msg::RawConfig => "Raw Config",
        Msg::RawConfigSaved => "Raw config saved",
        Msg::RawConfigSavedApplied => "Raw config saved and applied",
        Msg::RecordsSaved => "Configuration saved",
        Msg::RecordsSavedApplied => "Configuration saved and applied",
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
