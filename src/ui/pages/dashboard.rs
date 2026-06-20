use leptos::prelude::*;
use leptos::task::spawn_local;
use thaw::{MessageBar, MessageBarBody, MessageBarIntent, MessageBarLayout, Tab, TabList};

mod auth;
mod notice;
mod tabs;

use crate::api_types::{
    BackupInfo, ConfigResponse, SaveRawRequest, SaveRecordsRequest, ServiceStatus,
    TestConfigRequest,
};
use crate::config::model::{
    AddressRecord, CnameRecord, DnsRecords, HostRecord, ServerRecord, ValidationIssue,
};
use crate::i18n::{Locale, Msg, t};
use crate::ui::api;
use crate::ui::components::confirm_dialog::ConfirmDialog;
use crate::ui::components::status_badge::StatusBadge;
use crate::ui::components::toolbar::Toolbar;
use crate::ui::pages::backups::BackupsPanel;
use crate::ui::pages::raw_editor::RawEditorPanel;
use crate::ui::tables::address_table::AddressTable;
use crate::ui::tables::cname_table::CnameTable;
use crate::ui::tables::host_record_table::HostRecordTable;
use crate::ui::tables::server_table::ServerTable;
use crate::ui::tables::{EditableRow, editable_rows, row_values};

use self::auth::{AuthGate, AuthMode, is_unauthorized};
use self::notice::NoticeMessage;
use self::tabs::{TAB_ADDRESS, TAB_BACKUPS, TAB_CNAME, TAB_HOST_RECORD, TAB_RAW, TAB_SERVER};

#[component]
pub fn dashboard_page() -> impl IntoView {
    let auth_mode = RwSignal::new(AuthMode::Loading);
    let password = RwSignal::new(String::new());
    let locale = RwSignal::new(Locale::default());
    let active_tab = RwSignal::new(String::from(TAB_ADDRESS));
    let busy = RwSignal::new(false);
    let message = RwSignal::new(None::<NoticeMessage>);
    let warnings = RwSignal::new(Vec::<ValidationIssue>::new());
    let service_status = RwSignal::new(ServiceStatus::default());
    let unmanaged_line_count = RwSignal::new(0usize);

    let address = RwSignal::new(Vec::<EditableRow<AddressRecord>>::new());
    let host_record = RwSignal::new(Vec::<EditableRow<HostRecord>>::new());
    let cname = RwSignal::new(Vec::<EditableRow<CnameRecord>>::new());
    let server = RwSignal::new(Vec::<EditableRow<ServerRecord>>::new());
    let raw_content = RwSignal::new(String::new());
    let backups = RwSignal::new(Vec::<BackupInfo>::new());
    let delete_backup_open = RwSignal::new(false);
    let deleting_backup_id = RwSignal::new(None::<String>);
    let message_visible = Signal::derive(move || message.with(|message| message.is_some()));
    let message_text = Signal::derive(move || {
        let locale = locale.get();
        message.with(|message| {
            message
                .as_ref()
                .map(|message| message.render(locale))
                .unwrap_or_default()
        })
    });

    let current_records = move || DnsRecords {
        address: address.with(|rows| row_values(rows)),
        host_record: host_record.with(|rows| row_values(rows)),
        cname: cname.with(|rows| row_values(rows)),
        server: server.with(|rows| row_values(rows)),
    };

    let apply_config_response = move |response: ConfigResponse| {
        address.set(editable_rows(response.records.address));
        host_record.set(editable_rows(response.records.host_record));
        cname.set(editable_rows(response.records.cname));
        server.set(editable_rows(response.records.server));
        unmanaged_line_count.set(response.unmanaged_line_count);
        warnings.set(response.warnings);
        service_status.set(response.service);
    };

    let clear_auth_form = move || {
        password.set(String::new());
    };

    let handle_error = move |error: String| {
        if is_unauthorized(&error) {
            clear_auth_form();
            auth_mode.set(AuthMode::Login);
            message.set(Some(NoticeMessage::Localized(Msg::LoginRequired)));
        } else {
            message.set(Some(NoticeMessage::Raw(error)));
        }
    };

    let load_all = move || {
        busy.set(true);
        spawn_local(async move {
            let config = api::get_config().await;
            let raw = api::get_raw_config().await;
            let backup_list = api::list_backups().await;

            match config {
                Ok(response) => {
                    apply_config_response(response);
                    auth_mode.set(AuthMode::Authenticated);
                    message.set(Some(NoticeMessage::Localized(Msg::ConfigRefreshed)));
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
        spawn_local(async move {
            let config = api::get_config().await;
            let raw = api::get_raw_config().await;
            let backup_list = api::list_backups().await;

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
                    locale.set(status.locale);
                    if status.configured {
                        if status.authenticated {
                            load_all();
                        } else {
                            auth_mode.set(AuthMode::Login);
                            busy.set(false);
                        }
                    } else {
                        clear_auth_form();
                        auth_mode.set(AuthMode::Setup);
                        busy.set(false);
                    }
                }
                Err(error) => {
                    message.set(Some(NoticeMessage::Raw(error)));
                    auth_mode.set(AuthMode::Login);
                    busy.set(false);
                }
            }
        });
    };

    let switch_locale = move || {
        let next = locale.get_untracked().next();
        locale.set(next);
        spawn_local(async move {
            let _ = api::set_locale(next).await;
        });
    };

    let submit_auth = move || {
        busy.set(true);
        message.set(None);
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
                Ok(_) => {
                    password.set(String::new());
                    auth_mode.set(AuthMode::Authenticated);
                    load_all();
                }
                Err(error) => {
                    message.set(Some(NoticeMessage::Raw(error)));
                    busy.set(false);
                }
            }
        });
    };

    let save_records = move |apply: bool| {
        busy.set(true);
        let records = current_records();
        spawn_local(async move {
            let response = api::save_records(SaveRecordsRequest { records, apply }).await;
            match response {
                Ok(response) => {
                    warnings.set(response.warnings);
                    message.set(Some(NoticeMessage::Localized(if apply {
                        Msg::RecordsSavedApplied
                    } else {
                        Msg::RecordsSaved
                    })));
                    sync_all_silent();
                }
                Err(error) => handle_error(error),
            }
            busy.set(false);
        });
    };

    let save_raw = move |apply: bool| {
        busy.set(true);
        let content = raw_content.get();
        spawn_local(async move {
            let response = api::save_raw_config(SaveRawRequest { content, apply }).await;
            match response {
                Ok(response) => {
                    warnings.set(response.warnings);
                    message.set(Some(NoticeMessage::Localized(if apply {
                        Msg::RawConfigSavedApplied
                    } else {
                        Msg::RawConfigSaved
                    })));
                    sync_all_silent();
                }
                Err(error) => handle_error(error),
            }
            busy.set(false);
        });
    };

    let test_raw = move || {
        busy.set(true);
        let content = raw_content.get();
        spawn_local(async move {
            match api::test_config(TestConfigRequest {
                content: Some(content),
            })
            .await
            {
                Ok(report) => {
                    let output = if report.stdout.trim().is_empty() {
                        report.stderr
                    } else {
                        report.stdout
                    };
                    message.set(Some(NoticeMessage::LocalizedDetail {
                        msg: Msg::TestPassed,
                        detail: output.trim().into(),
                    }));
                }
                Err(error) => handle_error(error),
            }
            busy.set(false);
        });
    };

    let refresh_backups = move || {
        spawn_local(async move {
            match api::list_backups().await {
                Ok(response) => backups.set(response),
                Err(error) => handle_error(error),
            }
        });
    };

    let restore_backup = move |id: String| {
        busy.set(true);
        spawn_local(async move {
            match api::restore_backup(id).await {
                Ok(_) => {
                    message.set(Some(NoticeMessage::Localized(Msg::RestoreApplied)));
                    load_all();
                }
                Err(error) => {
                    handle_error(error);
                    busy.set(false);
                }
            }
        });
    };

    let request_delete_backup = move |id: String| {
        deleting_backup_id.set(Some(id));
        delete_backup_open.set(true);
    };

    let cancel_delete_backup = move || {
        deleting_backup_id.set(None);
        delete_backup_open.set(false);
    };

    let delete_backup = move || {
        let Some(id) = deleting_backup_id.update_untracked(Option::take) else {
            delete_backup_open.set(false);
            return;
        };
        busy.set(true);
        delete_backup_open.set(false);
        spawn_local(async move {
            match api::delete_backup(id).await {
                Ok(()) => {
                    message.set(Some(NoticeMessage::Localized(Msg::BackupDeleted)));
                    match api::list_backups().await {
                        Ok(response) => backups.set(response),
                        Err(error) => handle_error(error),
                    }
                }
                Err(error) => handle_error(error),
            }
            busy.set(false);
        });
    };

    let save_current = move || {
        if active_tab.with(|tab| tab == TAB_RAW) {
            save_raw(false);
        } else {
            save_records(false);
        }
    };
    let apply_current = move || {
        if active_tab.with(|tab| tab == TAB_RAW) {
            save_raw(true);
        } else {
            save_records(true);
        }
    };

    let logout = move || {
        clear_auth_form();
        auth_mode.set(AuthMode::Login);
        message.set(None);
        spawn_local(async move {
            let _ = api::logout().await;
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
                        message_visible=message_visible
                        message_text=message_text
                        locale=locale.into()
                        on_submit=move |_| submit_auth()
                        on_toggle_locale=move |_| switch_locale()
                    />
                }
            >
                <Toolbar
                    title="dnsmasq-web"
                    on_refresh=move |_| load_all()
                    on_save=move |_| save_current()
                    on_apply=move |_| apply_current()
                    on_logout=move |_| logout()
                    busy=busy.into()
                    locale=locale.into()
                    on_toggle_locale=move |_| switch_locale()
                />

                <div class="status-row">
                    <StatusBadge status=service_status.into() locale=locale.into() />
                    <span class="muted">
                        {move || format!("{}: {}", t(locale.get(), Msg::UnmanagedLines), unmanaged_line_count.get())}
                    </span>
                </div>

                <div class="alerts">
                    <Show when=move || message_visible.get()>
                        <MessageBar>
                            <MessageBarBody>{move || message_text.get()}</MessageBarBody>
                        </MessageBar>
                    </Show>

                    <Show when=move || warnings.with(|warnings| !warnings.is_empty())>
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
                    <Show when=move || active_tab.with(|tab| tab == TAB_ADDRESS)>
                        <AddressTable records=address locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.with(|tab| tab == TAB_HOST_RECORD)>
                        <HostRecordTable records=host_record locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.with(|tab| tab == TAB_CNAME)>
                        <CnameTable records=cname locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.with(|tab| tab == TAB_SERVER)>
                        <ServerTable records=server locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.with(|tab| tab == TAB_RAW)>
                        <RawEditorPanel content=raw_content on_test=move |_| test_raw() locale=locale.into() />
                    </Show>
                    <Show when=move || active_tab.with(|tab| tab == TAB_BACKUPS)>
                        <BackupsPanel
                            backups=backups.into()
                            on_refresh=move |_| refresh_backups()
                            on_restore=move |id| restore_backup(id)
                            on_delete=move |id| request_delete_backup(id)
                            locale=locale.into()
                        />
                    </Show>
                </main>

                <ConfirmDialog
                    open=delete_backup_open
                    message=Signal::derive(move || t(locale.get(), Msg::BackupDeleteConfirm).into())
                    on_confirm=move |_| delete_backup()
                    on_cancel=move |_| cancel_delete_backup()
                    locale=locale.into()
                />
            </Show>
        </div>
    }
}
