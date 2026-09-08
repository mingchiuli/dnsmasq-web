pub mod address_table;
pub mod cname_table;
pub mod host_record_table;
pub mod server_table;

use leptos::prelude::{ArcRwSignal, GetUntracked, RwSignal, Set, Update, With};
use std::cell::Cell;

thread_local! {
    static NEXT_EDITABLE_ROW_ID: Cell<u64> = const { Cell::new(1) };
}

#[derive(Clone)]
pub struct EditableRow<T> {
    pub id: u64,
    // Draft records outlive the tab owner that created or displayed them.
    pub value: ArcRwSignal<T>,
}

impl<T: Send + Sync + 'static> EditableRow<T> {
    pub fn new(value: T) -> Self {
        Self {
            id: next_editable_row_id(),
            value: ArcRwSignal::new(value),
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
) -> Option<ArcRwSignal<T>> {
    rows.with(|items| {
        items
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.value.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::Owner;

    #[test]
    fn draft_records_survive_tab_disposal_and_remain_editable() {
        let workspace = Owner::new();
        let rows = workspace.with(|| RwSignal::new(Vec::<EditableRow<String>>::new()));
        let tab = workspace.with(Owner::new);
        tab.with(|| upsert_row(rows, None, String::from("draft")));
        let id = rows.with(|rows| rows[0].id);
        tab.with(|| {
            let view_value = RwSignal::from(find_row(rows, id).expect("draft row"));
            assert_eq!(view_value.get_untracked(), "draft");
        });
        tab.cleanup();

        assert_eq!(rows.with(|rows| row_values(rows)), vec!["draft"]);
        workspace.with(|| upsert_row(rows, Some(id), String::from("edited")));
        assert_eq!(rows.with(|rows| rows[0].id), id);
        assert_eq!(rows.with(|rows| row_values(rows)), vec!["edited"]);
        remove_row(rows, id);
        assert!(rows.with(Vec::is_empty));
        workspace.cleanup();
    }
}
