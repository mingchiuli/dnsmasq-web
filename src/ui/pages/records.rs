use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::{
    Button, ButtonAppearance, ButtonType, Field, Input, InputType, MessageBar, MessageBarBody,
    MessageBarIntent, MessageBarLayout, Tab, TabList,
};

use crate::api_types::{
    BackupInfo, ConfigResponse, SaveRawRequest, SaveRecordsRequest, TestConfigRequest,
};
use crate::config::model::{
    AddressRecord, CnameRecord, DnsRecords, HostRecord, ServerRecord, ValidationIssue,
};
use crate::i18n::{Locale, Msg, t};
use crate::ui::api;
use crate::ui::components::status_badge::StatusBadge;
use crate::ui::components::toolbar::Toolbar;
use crate::ui::pages::backups::BackupsPanel;
use crate::ui::pages::raw_editor::RawEditorPanel;
use crate::ui::tables::address_table::AddressTable;
use crate::ui::tables::cname_table::CnameTable;
use crate::ui::tables::host_record_table::HostRecordTable;
use crate::ui::tables::server_table::ServerTable;
use crate::ui::tables::{EditableRecord, dns_records, editable_records};
use crate::ui::text::localized;

const SESSION_STORAGE_KEY: &str = "dnsmasqweb_session";

const TAB_ADDRESS: &str = "address";
const TAB_HOST_RECORD: &str = "host-record";
const TAB_CNAME: &str = "cname";
const TAB_SERVER: &str = "server";
const TAB_RAW: &str = "raw";
const TAB_BACKUPS: &str = "backups";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthMode {
    Loading,
    Setup,
    Login,
    Authenticated,
}

#[component]
pub fn RecordsPage() -> impl IntoView {
    let token = RwSignal::new(load_session_token());
    let auth_mode = RwSignal::new(AuthMode::Loading);
    let password = RwSignal::new(String::new());
    let locale = RwSignal::new(load_locale());
    let active_tab = RwSignal::new(String::from(TAB_ADDRESS));
    let busy = RwSignal::new(false);
    let message = RwSignal::new(String::new());
    let warnings = RwSignal::new(Vec::<ValidationIssue>::new());
    let service_status = RwSignal::new(crate::api_types::ServiceStatus::default());
    let unmanaged_line_count = RwSignal::new(0usize);

    let address = RwSignal::new(Vec::<EditableRecord<AddressRecord>>::new());
    let host_record = RwSignal::new(Vec::<EditableRecord<HostRecord>>::new());
    let cname = RwSignal::new(Vec::<EditableRecord<CnameRecord>>::new());
    let server = RwSignal::new(Vec::<EditableRecord<ServerRecord>>::new());
    let raw_content = RwSignal::new(String::new());
    let backups = RwSignal::new(Vec::<BackupInfo>::new());

    let current_records = move || DnsRecords {
        address: dns_records(&address.get()),
        host_record: dns_records(&host_record.get()),
        cname: dns_records(&cname.get()),
        server: dns_records(&server.get()),
    };

    let apply_config_response = move |response: ConfigResponse| {
        address.set(editable_records(response.records.address));
        host_record.set(editable_records(response.records.host_record));
        cname.set(editable_records(response.records.cname));
        server.set(editable_records(response.records.server));
        unmanaged_line_count.set(response.unmanaged_line_count);
        warnings.set(response.warnings);
        service_status.set(response.service);
    };

    let clear_session = move || {
        token.set(None);
        save_session_token(None);
        password.set(String::new());
    };

    let handle_error = move |error: String| {
        if is_unauthorized(&error) {
            clear_session();
            auth_mode.set(AuthMode::Login);
            message.set(t(locale.get_untracked(), Msg::LoginRequired).into());
        } else {
            message.set(error);
        }
    };

    let load_all = move || {
        busy.set(true);
        let token_value = token.get();
        spawn_local(async move {
            let config = api::get_config(token_value.clone()).await;
            let raw = api::get_raw_config(token_value.clone()).await;
            let backup_list = api::list_backups(token_value).await;

            match config {
                Ok(response) => {
                    apply_config_response(response);
                    auth_mode.set(AuthMode::Authenticated);
                    message.set(t(locale.get_untracked(), Msg::ConfigRefreshed).into());
                }
                Err(error) => handle_error(error),
            }
            if let Ok(response) = raw {
                raw_content.set(response.content);
            }
            if let Ok(response) = backup_list {
                backups.set(response);
            }
            busy.set(false);
        });
    };

    let sync_all_silent = move || {
        let token_value = token.get();
        spawn_local(async move {
            let config = api::get_config(token_value.clone()).await;
            let raw = api::get_raw_config(token_value.clone()).await;
            let backup_list = api::list_backups(token_value).await;

            match config {
                Ok(response) => apply_config_response(response),
                Err(error) => handle_error(error),
            }
            if let Ok(response) = raw {
                raw_content.set(response.content);
            }
            if let Ok(response) = backup_list {
                backups.set(response);
            }
        });
    };

    let check_auth_status = move || {
        busy.set(true);
        spawn_local(async move {
            match api::auth_status().await {
                Ok(status) => {
                    if status.configured {
                        if token.get_untracked().is_some() {
                            load_all();
                        } else {
                            auth_mode.set(AuthMode::Login);
                            busy.set(false);
                        }
                    } else {
                        clear_session();
                        auth_mode.set(AuthMode::Setup);
                        busy.set(false);
                    }
                }
                Err(error) => {
                    message.set(error);
                    auth_mode.set(AuthMode::Login);
                    busy.set(false);
                }
            }
        });
    };

    let submit_auth = move || {
        busy.set(true);
        message.set(String::new());
        let password_value = password.get();
        let mode = auth_mode.get_untracked();
        spawn_local(async move {
            let response = match mode {
                AuthMode::Setup => api::setup_password(password_value).await,
                AuthMode::Login => api::login(password_value).await,
                _ => {
                    busy.set(false);
                    return;
                }
            };

            match response {
                Ok(response) => {
                    save_session_token(Some(&response.token));
                    token.set(Some(response.token));
                    password.set(String::new());
                    auth_mode.set(AuthMode::Authenticated);
                    load_all();
                }
                Err(error) => {
                    message.set(error);
                    busy.set(false);
                }
            }
        });
    };

    let save_records = move |apply: bool| {
        busy.set(true);
        let token_value = token.get();
        let records = current_records();
        spawn_local(async move {
            let response =
                api::save_records(token_value, SaveRecordsRequest { records, apply }).await;
            match response {
                Ok(response) => {
                    warnings.set(response.warnings);
                    message.set(if apply {
                        t(locale.get_untracked(), Msg::RecordsSavedApplied).into()
                    } else {
                        t(locale.get_untracked(), Msg::RecordsSaved).into()
                    });
                    sync_all_silent();
                }
                Err(error) => handle_error(error),
            }
            busy.set(false);
        });
    };

    let save_raw = move |apply: bool| {
        busy.set(true);
        let token_value = token.get();
        let content = raw_content.get();
        spawn_local(async move {
            let response =
                api::save_raw_config(token_value, SaveRawRequest { content, apply }).await;
            match response {
                Ok(response) => {
                    warnings.set(response.warnings);
                    message.set(if apply {
                        t(locale.get_untracked(), Msg::RawConfigSavedApplied).into()
                    } else {
                        t(locale.get_untracked(), Msg::RawConfigSaved).into()
                    });
                    sync_all_silent();
                }
                Err(error) => handle_error(error),
            }
            busy.set(false);
        });
    };

    let test_raw = move || {
        busy.set(true);
        let token_value = token.get();
        let content = raw_content.get();
        spawn_local(async move {
            match api::test_config(
                token_value,
                TestConfigRequest {
                    content: Some(content),
                },
            )
            .await
            {
                Ok(report) => {
                    let output = if report.stdout.trim().is_empty() {
                        report.stderr
                    } else {
                        report.stdout
                    };
                    message.set(format!(
                        "{}: {}",
                        t(locale.get_untracked(), Msg::TestPassed),
                        output.trim()
                    ));
                }
                Err(error) => handle_error(error),
            }
            busy.set(false);
        });
    };

    let refresh_backups = move || {
        let token_value = token.get();
        spawn_local(async move {
            match api::list_backups(token_value).await {
                Ok(response) => backups.set(response),
                Err(error) => handle_error(error),
            }
        });
    };

    let restore_backup = move |id: String| {
        busy.set(true);
        let token_value = token.get();
        spawn_local(async move {
            match api::restore_backup(token_value, id).await {
                Ok(_) => {
                    message.set(t(locale.get_untracked(), Msg::RestoreApplied).into());
                    load_all();
                }
                Err(error) => {
                    handle_error(error);
                    busy.set(false);
                }
            }
        });
    };

    let save_current = move || match active_tab.get().as_str() {
        TAB_RAW => save_raw(false),
        _ => save_records(false),
    };
    let apply_current = move || match active_tab.get().as_str() {
        TAB_RAW => save_raw(true),
        _ => save_records(true),
    };

    let logout = move || {
        let token_value = token.get();
        clear_session();
        auth_mode.set(AuthMode::Login);
        message.set(String::new());
        spawn_local(async move {
            let _ = api::logout(token_value).await;
        });
    };

    Effect::new(move |_| {
        check_auth_status();
    });

    view! {
        <div class="app-shell">
            <Show
                when=move || auth_mode.get() == AuthMode::Authenticated
                fallback=move || view! {
                    <AuthGate
                        mode=auth_mode.into()
                        password=password
                        busy=busy.into()
                        message=message.into()
                        locale=locale.into()
                        on_submit=move |_| submit_auth()
                        on_toggle_locale=move |_| {
                            let next = locale.get_untracked().next();
                            save_locale(next);
                            locale.set(next);
                        }
                    />
                }
            >
                <Toolbar
                    title="dnsmasqweb"
                    on_refresh=move |_| load_all()
                    on_save=move |_| save_current()
                    on_apply=move |_| apply_current()
                    on_logout=move |_| logout()
                    busy=busy.into()
                    locale=locale.into()
                    on_toggle_locale=move |_| {
                        let next = locale.get_untracked().next();
                        save_locale(next);
                        locale.set(next);
                    }
                />

                <div class="status-row">
                    <StatusBadge status=service_status.into() locale=locale.into() />
                    <span class="muted">
                        {move || format!("{}: {}", t(locale.get(), Msg::UnmanagedLines), unmanaged_line_count.get())}
                    </span>
                </div>

                <div class="alerts">
                    <Show when=move || !message.get().is_empty()>
                        <MessageBar>
                            <MessageBarBody>{move || message.get()}</MessageBarBody>
                        </MessageBar>
                    </Show>

                    <Show when=move || !warnings.get().is_empty()>
                        <MessageBar intent=MessageBarIntent::Warning layout=MessageBarLayout::Multiline>
                            <MessageBarBody>
                            <For
                                each=move || warnings.get()
                                key=|issue| issue.message.clone()
                                children=|issue| view! { <div>{issue.message}</div> }
                            />
                            </MessageBarBody>
                        </MessageBar>
                    </Show>
                </div>

                <TabList class="tabs" selected_value=active_tab>
                    <Tab value=TAB_ADDRESS>{move || t(locale.get(), Msg::Address)}</Tab>
                    <Tab value=TAB_HOST_RECORD>{move || t(locale.get(), Msg::HostRecord)}</Tab>
                    <Tab value=TAB_CNAME>{move || t(locale.get(), Msg::Cname)}</Tab>
                    <Tab value=TAB_SERVER>{move || t(locale.get(), Msg::Server)}</Tab>
                    <Tab value=TAB_RAW>{move || t(locale.get(), Msg::RawConfig)}</Tab>
                    <Tab value=TAB_BACKUPS>{move || t(locale.get(), Msg::Backups)}</Tab>
                </TabList>

                <main class="content">
                    <Show when=move || active_tab.get() == TAB_ADDRESS>
                        <AddressTable records=address locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.get() == TAB_HOST_RECORD>
                        <HostRecordTable records=host_record locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.get() == TAB_CNAME>
                        <CnameTable records=cname locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.get() == TAB_SERVER>
                        <ServerTable records=server locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.get() == TAB_RAW>
                        <RawEditorPanel content=raw_content on_test=move |_| test_raw() locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.get() == TAB_BACKUPS>
                        <BackupsPanel
                            backups=backups.into()
                            on_refresh=move |_| refresh_backups()
                            on_restore=move |id| restore_backup(id)
                            locale=locale.into()
                        />
                    </Show>
                </main>
            </Show>
        </div>
    }
}

#[component]
fn AuthGate(
    mode: Signal<AuthMode>,
    password: RwSignal<String>,
    busy: Signal<bool>,
    message: Signal<String>,
    locale: Signal<Locale>,
    #[prop(into)] on_submit: Callback<()>,
    #[prop(into)] on_toggle_locale: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="auth-shell">
            <div class="auth-head">
                <h1>"dnsmasqweb"</h1>
                <Button button_type=ButtonType::Button on_click=move |_| on_toggle_locale.run(())>
                    {move || t(locale.get(), Msg::LocaleSwitch)}
                </Button>
            </div>
            <form
                class="auth-panel"
                on:submit=move |ev| {
                    ev.prevent_default();
                    on_submit.run(());
                }
            >
                <h2>{move || match mode.get() {
                    AuthMode::Setup => t(locale.get(), Msg::SetupPassword),
                    AuthMode::Login => t(locale.get(), Msg::Login),
                    AuthMode::Loading | AuthMode::Authenticated => t(locale.get(), Msg::Loading),
                }}</h2>
                <Show when=move || mode.get() != AuthMode::Loading>
                    <Field label=localized(locale, Msg::Password)>
                        <Input
                            value=password
                            input_type=InputType::Password
                            autocomplete="current-password"
                        />
                    </Field>
                    <Button
                        appearance=ButtonAppearance::Primary
                        button_type=ButtonType::Submit
                        disabled=busy
                    >
                        {move || match mode.get() {
                            AuthMode::Setup => t(locale.get(), Msg::SetPassword),
                            AuthMode::Login => t(locale.get(), Msg::Login),
                            AuthMode::Loading | AuthMode::Authenticated => t(locale.get(), Msg::Loading),
                        }}
                    </Button>
                </Show>
                <Show when=move || !message.get().is_empty()>
                    <MessageBar intent=MessageBarIntent::Error layout=MessageBarLayout::Multiline>
                        <MessageBarBody>{move || message.get()}</MessageBarBody>
                    </MessageBar>
                </Show>
            </form>
        </div>
    }
}

fn load_locale() -> Locale {
    let Some(window) = web_sys::window() else {
        return Locale::default();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Locale::default();
    };
    storage
        .get_item("dnsmasqweb_locale")
        .ok()
        .flatten()
        .as_deref()
        .map(Locale::from)
        .unwrap_or_default()
}

fn save_locale(locale: Locale) {
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.set_item("dnsmasqweb_locale", locale.code());
    }
}

fn load_session_token() -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(SESSION_STORAGE_KEY).ok().flatten()
}

fn save_session_token(token: Option<&str>) {
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        match token {
            Some(token) => {
                let _ = storage.set_item(SESSION_STORAGE_KEY, token);
            }
            None => {
                let _ = storage.remove_item(SESSION_STORAGE_KEY);
            }
        }
    }
}

fn is_unauthorized(error: &str) -> bool {
    error.contains("unauthorized")
}
