use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api_types::{BootstrapResponse, DashboardBootstrap};
use crate::i18n::{Locale, Msg};
use crate::ui::api;

use super::auth::{AuthMode, AuthState, is_unauthorized};
use super::backup_actions::BackupState;
use super::notice::NoticeState;
use super::tabs::TAB_ADDRESS;
use super::workspace::WorkspaceState;

#[derive(Clone, Copy)]
pub(super) struct DashboardController {
    pub(super) auth: AuthState,
    pub(super) backups: BackupState,
    pub(super) workspace: WorkspaceState,
    pub(super) notice: NoticeState,
    pub(super) locale: RwSignal<Locale>,
    pub(super) active_tab: RwSignal<String>,
    pub(super) busy: RwSignal<bool>,
}

impl DashboardController {
    pub(super) fn new(initial: BootstrapResponse) -> Self {
        let controller = Self {
            auth: AuthState::new(),
            backups: BackupState::new(),
            workspace: WorkspaceState::new(),
            notice: NoticeState::new(),
            locale: RwSignal::new(Locale::default()),
            active_tab: RwSignal::new(String::from(TAB_ADDRESS)),
            busy: RwSignal::new(false),
        };
        controller.apply_bootstrap(initial, false);
        controller
    }

    pub(super) fn load_all(self) {
        self.busy.set(true);
        spawn_local(async move {
            match api::bootstrap().await {
                Ok(response) => self.apply_bootstrap(response, true),
                Err(error) => self.handle_error(error),
            }
            self.busy.set(false);
        });
    }

    pub(super) fn sync_all_silent(self) {
        spawn_local(async move {
            match api::bootstrap().await {
                Ok(response) => self.apply_bootstrap(response, false),
                Err(error) => self.handle_error(error),
            }
        });
    }

    pub(super) fn switch_locale(self) {
        let next = self.locale.get_untracked().next();
        self.locale.set(next);
        spawn_local(async move {
            let _ = api::set_locale(next).await;
        });
    }

    pub(super) fn handle_error(self, error: String) {
        if is_unauthorized(&error) {
            self.auth.clear_login_form();
            self.auth.mode.set(AuthMode::Login);
            self.notice.show_localized(Msg::LoginRequired);
        } else {
            self.notice.show_raw(error);
        }
    }

    fn apply_bootstrap(self, response: BootstrapResponse, announce: bool) {
        match response {
            BootstrapResponse::Setup { locale } => {
                self.locale.set(locale);
                self.clear_dashboard();
                self.auth.mode.set(AuthMode::Setup);
            }
            BootstrapResponse::Login { locale } => {
                self.locale.set(locale);
                self.clear_dashboard();
                self.auth.mode.set(AuthMode::Login);
            }
            BootstrapResponse::Authenticated { locale, dashboard } => {
                self.locale.set(locale);
                self.auth.mode.set(AuthMode::Authenticated);
                self.apply_dashboard_bootstrap(*dashboard, announce);
            }
        }
    }

    fn apply_dashboard_bootstrap(self, dashboard: DashboardBootstrap, announce: bool) {
        let mut errors = Vec::new();
        match dashboard.config {
            Ok(response) => self.workspace.apply_config(response),
            Err(error) => errors.push(error),
        }
        match dashboard.raw {
            Ok(response) => self.workspace.apply_raw(response),
            Err(error) => errors.push(error),
        }
        match dashboard.backups {
            Ok(response) => self.backups.items.set(response),
            Err(error) => errors.push(error),
        }

        if errors.is_empty() {
            if announce {
                self.notice.show_localized(Msg::ConfigRefreshed);
            }
        } else {
            self.notice.show_raw(errors.join("; "));
        }
    }

    pub(super) fn clear_dashboard(self) {
        self.workspace.clear();
        self.backups.clear();
    }
}
