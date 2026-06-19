pub mod address_table;
pub mod cname_table;
pub mod host_record_table;
pub mod server_table;

use leptos::prelude::{GetUntracked, RwSignal, Set, Update, With};
use std::cell::Cell;

thread_local! {
    static NEXT_EDITABLE_ROW_ID: Cell<u64> = const { Cell::new(1) };
}

#[derive(Clone)]
pub struct EditableRow<T> {
    pub id: u64,
    pub value: RwSignal<T>,
}

impl<T: Send + Sync + 'static> EditableRow<T> {
    pub fn new(value: T) -> Self {
        Self {
            id: next_editable_row_id(),
            value: RwSignal::new(value),
        }
    }
}

fn next_editable_row_id() -> u64 {
    NEXT_EDITABLE_ROW_ID.with(|next| {
        let id = next.get();
        next.set(id + 1);
        id
    })
}

pub fn editable_rows<T: Send + Sync + 'static>(values: Vec<T>) -> Vec<EditableRow<T>> {
    values.into_iter().map(EditableRow::new).collect()
}

pub fn row_values<T: Clone + Send + Sync + 'static>(rows: &[EditableRow<T>]) -> Vec<T> {
    rows.iter().map(|row| row.value.get_untracked()).collect()
}

pub fn find_row<T: Send + Sync + 'static>(
    rows: RwSignal<Vec<EditableRow<T>>>,
    id: u64,
) -> Option<RwSignal<T>> {
    rows.with(|items| {
        items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.value)
    })
}

pub fn remove_row<T: Send + Sync + 'static>(rows: RwSignal<Vec<EditableRow<T>>>, id: u64) {
    rows.update(|items| items.retain(|item| item.id != id));
}

pub fn upsert_row<T: Send + Sync + 'static>(
    rows: RwSignal<Vec<EditableRow<T>>>,
    id: Option<u64>,
    value: T,
) {
    rows.update(|items| {
        if let Some(id) = id
            && let Some(item) = items.iter_mut().find(|item| item.id == id)
        {
            item.value.set(value);
            return;
        }
        items.push(EditableRow::new(value));
    });
}
