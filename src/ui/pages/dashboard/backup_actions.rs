use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api_types::BackupInfo;
use crate::i18n::Msg;
use crate::ui::api;

use super::controller::DashboardController;

#[derive(Clone, Copy)]
pub(super) struct BackupState {
    pub(super) items: RwSignal<Vec<BackupInfo>>,
    pub(super) delete_dialog_open: RwSignal<bool>,
    deleting_id: RwSignal<Option<String>>,
}

impl BackupState {
    pub(super) fn new() -> Self {
        Self {
            items: RwSignal::new(Vec::new()),
            delete_dialog_open: RwSignal::new(false),
            deleting_id: RwSignal::new(None),
        }
    }

    pub(super) fn clear(self) {
        self.items.set(Vec::new());
        self.deleting_id.set(None);
        self.delete_dialog_open.set(false);
    }
}

impl DashboardController {
    pub(super) fn refresh_backups(self) {
        spawn_local(async move {
            match api::list_backups().await {
                Ok(response) => self.backups.items.set(response),
                Err(error) => self.handle_error(error),
            }
        });
    }

    pub(super) fn restore_backup(self, id: String) {
        self.busy.set(true);
        spawn_local(async move {
            match api::restore_backup(id).await {
                Ok(_) => {
                    self.notice.show_localized(Msg::RestoreApplied);
                    self.load_all();
                }
                Err(error) => {
                    self.handle_error(error);
                    self.busy.set(false);
                }
            }
        });
    }

    pub(super) fn request_delete_backup(self, id: String) {
        self.backups.deleting_id.set(Some(id));
        self.backups.delete_dialog_open.set(true);
    }

    pub(super) fn cancel_delete_backup(self) {
        self.backups.deleting_id.set(None);
        self.backups.delete_dialog_open.set(false);
    }

    pub(super) fn delete_backup(self) {
        let Some(id) = self.backups.deleting_id.update_untracked(Option::take) else {
            self.backups.delete_dialog_open.set(false);
            return;
        };
        self.busy.set(true);
        self.backups.delete_dialog_open.set(false);
        spawn_local(async move {
            match api::delete_backup(id).await {
                Ok(()) => {
                    self.notice.show_localized(Msg::BackupDeleted);
                    match api::list_backups().await {
                        Ok(response) => self.backups.items.set(response),
                        Err(error) => self.handle_error(error),
                    }
                }
                Err(error) => self.handle_error(error),
            }
            self.busy.set(false);
        });
    }
}
