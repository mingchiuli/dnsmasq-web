pub mod address_table;
pub mod cname_table;
pub mod host_record_table;
pub mod server_table;

use leptos::prelude::{RwSignal, Update};
use std::cell::Cell;

thread_local! {
    static NEXT_EDITABLE_RECORD_ID: Cell<u64> = const { Cell::new(1) };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableRecord<T> {
    pub id: u64,
    pub record: T,
}

impl<T> EditableRecord<T> {
    pub fn new(record: T) -> Self {
        Self {
            id: next_editable_record_id(),
            record,
        }
    }
}

fn next_editable_record_id() -> u64 {
    NEXT_EDITABLE_RECORD_ID.with(|next| {
        let id = next.get();
        next.set(id + 1);
        id
    })
}

pub fn editable_records<T>(records: Vec<T>) -> Vec<EditableRecord<T>> {
    records.into_iter().map(EditableRecord::new).collect()
}

pub fn dns_records<T: Clone>(records: &[EditableRecord<T>]) -> Vec<T> {
    records.iter().map(|row| row.record.clone()).collect()
}

pub fn remove_record<T: Send + Sync + 'static>(records: RwSignal<Vec<EditableRecord<T>>>, id: u64) {
    records.update(|items| items.retain(|item| item.id != id));
}

pub fn upsert_record<T: Send + Sync + 'static>(
    records: RwSignal<Vec<EditableRecord<T>>>,
    id: Option<u64>,
    record: T,
) {
    records.update(|items| {
        if let Some(id) = id
            && let Some(item) = items.iter_mut().find(|item| item.id == id)
        {
            item.record = record;
            return;
        }
        items.push(EditableRecord::new(record));
    });
}
