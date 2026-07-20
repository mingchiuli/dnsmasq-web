use leptos::{html, prelude::*};

#[slot]
pub struct ModalActions {
    children: Children,
}

#[component]
pub fn modal(
    #[prop(into)] open: Signal<bool>,
    title: Signal<String>,
    #[prop(into)] on_dismiss: Callback<()>,
    modal_actions: ModalActions,
    children: Children,
) -> impl IntoView {
    let dialog_ref = NodeRef::<html::Dialog>::new();

    #[cfg(not(feature = "hydrate"))]
    let _ = open;

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let should_open = open.get();
        let Some(dialog) = dialog_ref.get() else {
            return;
        };

        if should_open && !dialog.open() {
            let _ = dialog.show_modal();
        } else if !should_open && dialog.open() {
            dialog.close();
        }
    });

    view! {
        <dialog
            class="ui-modal"
            aria-label=move || title.get()
            node_ref=dialog_ref
            on:cancel=move |event: leptos::ev::Event| {
                event.prevent_default();
                on_dismiss.run(());
            }
            on:click=move |event| {
                if event.target() == event.current_target() {
                    on_dismiss.run(());
                }
            }
        >
            <div class="ui-modal__content">
                <h2 class="ui-modal__title">{move || title.get()}</h2>
                <div class="ui-modal__body">{children()}</div>
                <div class="ui-modal__actions">{(modal_actions.children)()}</div>
            </div>
        </dialog>
    }
    .into_any()
}
