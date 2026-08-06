use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api_types::{
    ConfigResponse, ConfigRevision, RawConfigResponse, SaveOutcome, SaveRawRequest,
    SaveRecordsRequest, ServiceStatus, TestConfigRequest,
};
use crate::config::model::{
    AddressRecord, CnameRecord, DnsRecords, HostRecord, ServerRecord, ValidationIssue,
};
use crate::i18n::Msg;
use crate::ui::api;
use crate::ui::tables::{EditableRow, editable_rows, row_values};

use super::controller::DashboardController;
use super::notice::NoticeMessage;
use super::tabs::TAB_RAW;

#[derive(Clone, Copy)]
pub(super) struct WorkspaceState {
    pub(super) address: RwSignal<Vec<EditableRow<AddressRecord>>>,
    pub(super) host_record: RwSignal<Vec<EditableRow<HostRecord>>>,
    pub(super) cname: RwSignal<Vec<EditableRow<CnameRecord>>>,
    pub(super) server: RwSignal<Vec<EditableRow<ServerRecord>>>,
    pub(super) raw_content: RwSignal<String>,
    structured_revision: RwSignal<Option<ConfigRevision>>,
    raw_revision: RwSignal<Option<ConfigRevision>>,
    pub(super) warnings: RwSignal<Vec<ValidationIssue>>,
    pub(super) service_status: RwSignal<ServiceStatus>,
    pub(super) unmanaged_line_count: RwSignal<usize>,
}

impl WorkspaceState {
    pub(super) fn new() -> Self {
        Self {
            address: RwSignal::new(Vec::new()),
            host_record: RwSignal::new(Vec::new()),
            cname: RwSignal::new(Vec::new()),
            server: RwSignal::new(Vec::new()),
            raw_content: RwSignal::new(String::new()),
            structured_revision: RwSignal::new(None),
            raw_revision: RwSignal::new(None),
            warnings: RwSignal::new(Vec::new()),
            service_status: RwSignal::new(ServiceStatus::default()),
            unmanaged_line_count: RwSignal::new(0),
        }
    }

    pub(super) fn clear(self) {
        self.address.set(Vec::new());
        self.host_record.set(Vec::new());
        self.cname.set(Vec::new());
        self.server.set(Vec::new());
        self.raw_content.set(String::new());
        self.structured_revision.set(None);
        self.raw_revision.set(None);
        self.warnings.set(Vec::new());
        self.service_status.set(ServiceStatus::default());
        self.unmanaged_line_count.set(0);
    }

    pub(super) fn apply_config(self, response: ConfigResponse) {
        self.structured_revision.set(Some(response.revision));
        self.address.set(editable_rows(response.records.address));
        self.host_record
            .set(editable_rows(response.records.host_record));
        self.cname.set(editable_rows(response.records.cname));
        self.server.set(editable_rows(response.records.server));
        self.unmanaged_line_count.set(response.unmanaged_line_count);
        self.warnings.set(response.warnings);
        self.service_status.set(response.service);
    }

    pub(super) fn apply_raw(self, response: RawConfigResponse) {
        self.raw_content.set(response.content);
        self.raw_revision.set(Some(response.revision));
    }

    fn records(self) -> DnsRecords {
        DnsRecords {
            address: self.address.with(|rows| row_values(rows)),
            host_record: self.host_record.with(|rows| row_values(rows)),
            cname: self.cname.with(|rows| row_values(rows)),
            server: self.server.with(|rows| row_values(rows)),
        }
    }
}

impl DashboardController {
    pub(super) fn save_current(self, apply: bool) {
        if self.active_tab.with(|tab| tab == TAB_RAW) {
            self.save_raw(apply);
        } else {
            self.save_records(apply);
        }
    }

    pub(super) fn test_raw(self) {
        self.busy.set(true);
        let content = self.workspace.raw_content.get();
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
                    self.notice.show(NoticeMessage::LocalizedDetail {
                        msg: Msg::TestPassed,
                        detail: output.trim().into(),
                    });
                }
                Err(error) => self.handle_error(error),
            }
            self.busy.set(false);
        });
    }

    fn save_records(self, apply: bool) {
        self.busy.set(true);
        let records = self.workspace.records();
        let revision = self.workspace.structured_revision.get();
        spawn_local(async move {
            let Some(revision) = revision else {
                self.notice.show_localized(Msg::ConfigChanged);
                self.busy.set(false);
                return;
            };
            let response = api::save_records(SaveRecordsRequest {
                records,
                apply,
                revision,
            })
            .await;
            match response {
                Ok(SaveOutcome::Saved(response)) => {
                    self.workspace.warnings.set(response.warnings);
                    self.notice.show_localized(if apply {
                        Msg::RecordsSavedApplied
                    } else {
                        Msg::RecordsSaved
                    });
                    self.sync_all_silent();
                }
                Ok(SaveOutcome::Conflict) => {
                    self.notice.show_localized(Msg::ConfigChanged);
                }
                Err(error) => self.handle_error(error),
            }
            self.busy.set(false);
        });
    }

    fn save_raw(self, apply: bool) {
        self.busy.set(true);
        let content = self.workspace.raw_content.get();
        let revision = self.workspace.raw_revision.get();
        spawn_local(async move {
            let Some(revision) = revision else {
                self.notice.show_localized(Msg::ConfigChanged);
                self.busy.set(false);
                return;
            };
            let response = api::save_raw_config(SaveRawRequest {
                content,
                apply,
                revision,
            })
            .await;
            match response {
                Ok(SaveOutcome::Saved(response)) => {
                    self.workspace.warnings.set(response.warnings);
                    self.notice.show_localized(if apply {
                        Msg::RawConfigSavedApplied
                    } else {
                        Msg::RawConfigSaved
                    });
                    self.sync_all_silent();
                }
                Ok(SaveOutcome::Conflict) => {
                    self.notice.show_localized(Msg::ConfigChanged);
                }
                Err(error) => self.handle_error(error),
            }
            self.busy.set(false);
        });
    }
}
