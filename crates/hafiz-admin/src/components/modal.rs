//! Modal dialog component

use leptos::*;

#[component]
pub fn Modal(
    title: &'static str,
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    children: Children,
    #[prop(optional)] size: Option<&'static str>,
) -> impl IntoView {
    let size_class = match size.unwrap_or("md") {
        "sm" => "max-w-md",
        "lg" => "max-w-2xl",
        "xl" => "max-w-4xl",
        _ => "max-w-lg",
    };

    view! {
        {move || show.get().then(|| view! {
            <div class="fixed inset-0 z-50 overflow-y-auto">
                // Backdrop
                <div
                    class="fixed inset-0 bg-black/60 transition-opacity"
                    on:click=move |_| on_close.call(())
                />

                // Modal
                <div class="flex min-h-full items-center justify-center p-4">
                    <div class=format!("relative w-full {} bg-gray-800 rounded-xl shadow-2xl border border-gray-700", size_class)>
                        // Header
                        <div class="flex items-center justify-between px-6 py-4 border-b border-gray-700">
                            <h3 class="text-lg font-semibold text-white">{title}</h3>
                            <button
                                class="text-gray-400 hover:text-white transition-colors"
                                on:click=move |_| on_close.call(())
                            >
                                <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        </div>

                        // Content
                        <div class="px-6 py-4">
                            {children()}
                        </div>
                    </div>
                </div>
            </div>
        })}
    }
}

/// Confirmation modal
#[component]
pub fn ConfirmModal(
    title: &'static str,
    message: String,
    show: ReadSignal<bool>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
    #[prop(optional)] confirm_text: Option<&'static str>,
    #[prop(optional)] danger: Option<bool>,
) -> impl IntoView {
    let confirm_btn_class = if danger.unwrap_or(false) {
        "bg-red-600 hover:bg-red-700"
    } else {
        "bg-blue-600 hover:bg-blue-700"
    };

    view! {
        <Modal title=title show=show on_close=on_cancel>
            <p class="text-gray-300 mb-6">{message}</p>
            <div class="flex justify-end space-x-3">
                <button
                    class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
                    on:click=move |_| on_cancel.call(())
                >
                    "Cancel"
                </button>
                <button
                    class=format!("px-4 py-2 {} text-white rounded-lg transition-colors", confirm_btn_class)
                    on:click=move |_| on_confirm.call(())
                >
                    {confirm_text.unwrap_or("Confirm")}
                </button>
            </div>
        </Modal>
    }
}
