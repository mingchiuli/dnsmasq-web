use leptos::prelude::*;

#[component]
pub fn EmptyTableMessage(message: Signal<&'static str>) -> impl IntoView {
    view! { <div class="empty-table">{move || message.get()}</div> }
}

#[component]
pub fn EditableTable(
    is_empty: Signal<bool>,
    empty_message: Signal<&'static str>,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <Show
            when=move || !is_empty.get()
            fallback=move || view! { <EmptyTableMessage message=empty_message /> }
        >
            <div class="record-table">{children()}</div>
        </Show>
    }
}
