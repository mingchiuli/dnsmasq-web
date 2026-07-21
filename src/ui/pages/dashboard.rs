use leptos::prelude::*;

mod auth;
mod backup_actions;
mod controller;
mod notice;
mod tabs;
mod workspace;

use crate::api_types::BootstrapResponse;
use crate::i18n::{Msg, t};
use crate::ui::api;
use crate::ui::components::confirm_dialog::ConfirmDialog;
use crate::ui::components::notice::{Notice, NoticeTone};
use crate::ui::components::status_badge::StatusBadge;
use crate::ui::components::toolbar::Toolbar;
use crate::ui::pages::backups::BackupsPanel;
use crate::ui::pages::raw_editor::RawEditorPanel;
use crate::ui::tables::address_table::AddressTable;
use crate::ui::tables::cname_table::CnameTable;
use crate::ui::tables::host_record_table::HostRecordTable;
use crate::ui::tables::server_table::ServerTable;

use self::auth::{AuthGate, AuthMode, ChangePasswordDialog};
use self::controller::DashboardController;
use self::tabs::{
    DashboardTabPanel, DashboardTabs, TAB_ADDRESS, TAB_BACKUPS, TAB_CNAME, TAB_HOST_RECORD,
    TAB_RAW, TAB_SERVER,
};

#[component]
pub fn dashboard_page() -> impl IntoView {
    let bootstrap = Resource::new_blocking(|| (), |_| api::bootstrap());

    view! {
        <Suspense fallback=|| ()>
            {move || bootstrap.get().map(|response| match response {
                Ok(initial) => view! { <Dashboard initial=initial /> }.into_any(),
                Err(error) => view! {
                    <div class="auth-shell">
                        <div class="auth-head"><h1>"dnsmasq-web"</h1></div>
                        <Notice tone=NoticeTone::Error multiline=true>{error}</Notice>
                    </div>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn dashboard(initial: BootstrapResponse) -> impl IntoView {
    let controller = DashboardController::new(initial);
    let auth = controller.auth;
    let backups = controller.backups;
    let workspace = controller.workspace;
    let locale = controller.locale;
    let active_tab = controller.active_tab;
    let busy = controller.busy;
    let message_visible = controller.notice.visible();
    let message_text = controller.notice.text(locale);

    view! {
        <div class="app-shell">
            <Show
                when=move || auth.mode.get() == AuthMode::Authenticated
                fallback=move || view! {
                    <AuthGate
                        mode=auth.mode.into()
                        password=auth.password
                        password_confirmation=auth.password_confirmation
                        busy=busy.into()
                        message_visible=message_visible
                        message_text=message_text
                        locale=locale.into()
                        on_submit=move |_| controller.submit_auth()
                        on_toggle_locale=move |_| controller.switch_locale()
                    />
                }
            >
                <Toolbar
                    title="dnsmasq-web"
                    on_refresh=move |_| controller.load_all()
                    on_save=move |_| controller.save_current(false)
                    on_apply=move |_| controller.save_current(true)
                    on_change_password=move |_| controller.open_change_password()
                    on_logout=move |_| controller.logout()
                    busy=busy.into()
                    locale=locale.into()
                    on_toggle_locale=move |_| controller.switch_locale()
                />

                <div class="status-row">
                    <StatusBadge status=workspace.service_status.into() locale=locale.into() />
                    <span class="muted">
                        {move || format!(
                            "{}: {}",
                            t(locale.get(), Msg::UnmanagedLines),
                            workspace.unmanaged_line_count.get(),
                        )}
                    </span>
                </div>

                <div class="alerts">
                    <Show when=move || message_visible.get()>
                        <Notice>{move || message_text.get()}</Notice>
                    </Show>

                    <Show when=move || workspace.warnings.with(|warnings| !warnings.is_empty())>
                        <Notice tone=NoticeTone::Warning multiline=true>
                            <For
                                each=move || workspace.warnings.get()
                                key=|issue| issue.message.clone()
                                children=|issue| view! { <div>{issue.message}</div> }
                            />
                        </Notice>
                    </Show>
                </div>

                <DashboardTabs active_tab=active_tab locale=locale.into() />

                <main class="content">
                    <DashboardTabPanel value=TAB_ADDRESS active_tab=active_tab.into()>
                        <Show when=move || active_tab.with(|tab| tab == TAB_ADDRESS)>
                            <AddressTable records=workspace.address locale=locale.into() />
                        </Show>
                    </DashboardTabPanel>
                    <DashboardTabPanel value=TAB_HOST_RECORD active_tab=active_tab.into()>
                        <Show when=move || active_tab.with(|tab| tab == TAB_HOST_RECORD)>
                            <HostRecordTable records=workspace.host_record locale=locale.into() />
                        </Show>
                    </DashboardTabPanel>
                    <DashboardTabPanel value=TAB_CNAME active_tab=active_tab.into()>
                        <Show when=move || active_tab.with(|tab| tab == TAB_CNAME)>
                            <CnameTable records=workspace.cname locale=locale.into() />
                        </Show>
                    </DashboardTabPanel>
                    <DashboardTabPanel value=TAB_SERVER active_tab=active_tab.into()>
                        <Show when=move || active_tab.with(|tab| tab == TAB_SERVER)>
                            <ServerTable records=workspace.server locale=locale.into() />
                        </Show>
                    </DashboardTabPanel>
                    <DashboardTabPanel value=TAB_RAW active_tab=active_tab.into()>
                        <Show when=move || active_tab.with(|tab| tab == TAB_RAW)>
                            <RawEditorPanel
                                content=workspace.raw_content
                                on_test=move |_| controller.test_raw()
                                locale=locale.into()
                            />
                        </Show>
                    </DashboardTabPanel>
                    <DashboardTabPanel value=TAB_BACKUPS active_tab=active_tab.into()>
                        <Show when=move || active_tab.with(|tab| tab == TAB_BACKUPS)>
                            <BackupsPanel
                                backups=backups.items.into()
                                on_refresh=move |_| controller.refresh_backups()
                                on_restore=move |id| controller.restore_backup(id)
                                on_delete=move |id| controller.request_delete_backup(id)
                                locale=locale.into()
                            />
                        </Show>
                    </DashboardTabPanel>
                </main>

                <ConfirmDialog
                    open=backups.delete_dialog_open
                    message=Signal::derive(move || {
                        t(locale.get(), Msg::BackupDeleteConfirm).into()
                    })
                    on_confirm=move |_| controller.delete_backup()
                    on_cancel=move |_| controller.cancel_delete_backup()
                    locale=locale.into()
                />

                <ChangePasswordDialog
                    open=auth.change_password_open
                    current_password=auth.current_password
                    new_password=auth.new_password
                    new_password_confirmation=auth.new_password_confirmation
                    busy=busy.into()
                    error=auth.change_password_error.into()
                    locale=locale.into()
                    on_save=move |_| controller.change_password()
                    on_cancel=move |_| controller.cancel_change_password()
                />
            </Show>
        </div>
    }
}
