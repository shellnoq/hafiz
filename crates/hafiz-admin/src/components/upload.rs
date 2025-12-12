//! File upload component with drag & drop support

use leptos::*;
use web_sys::{DragEvent, File, FileList, HtmlInputElement};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

/// Upload state for tracking progress
#[derive(Clone, Debug)]
pub struct UploadItem {
    pub id: u32,
    pub name: String,
    pub size: u64,
    pub progress: f64,
    pub status: UploadStatus,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Complete,
    Error,
}

/// File upload modal component
#[component]
pub fn FileUploadModal(
    bucket: String,
    prefix: String,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] on_upload_complete: Callback<()>,
) -> impl IntoView {
    let (is_dragging, set_dragging) = create_signal(false);
    let (uploads, set_uploads) = create_signal::<Vec<UploadItem>>(vec![]);
    let (upload_counter, set_upload_counter) = create_signal(0u32);
    let (is_uploading, set_is_uploading) = create_signal(false);

    let file_input_ref = create_node_ref::<leptos::html::Input>();

    // Handle drag events
    let on_dragenter = move |ev: DragEvent| {
        ev.prevent_default();
        set_dragging.set(true);
    };

    let on_dragleave = move |ev: DragEvent| {
        ev.prevent_default();
        set_dragging.set(false);
    };

    let on_dragover = move |ev: DragEvent| {
        ev.prevent_default();
    };

    let bucket_clone = bucket.clone();
    let prefix_clone = prefix.clone();

    // Process dropped files
    let process_files = move |files: FileList| {
        let mut new_uploads = vec![];
        let count = files.length();

        for i in 0..count {
            if let Some(file) = files.get(i) {
                let id = upload_counter.get() + i;
                new_uploads.push(UploadItem {
                    id,
                    name: file.name(),
                    size: file.size() as u64,
                    progress: 0.0,
                    status: UploadStatus::Pending,
                    error: None,
                });
            }
        }

        set_upload_counter.update(|c| *c += count);
        set_uploads.update(|u| u.extend(new_uploads));
    };

    let on_drop = {
        let process_files = process_files.clone();
        move |ev: DragEvent| {
            ev.prevent_default();
            set_dragging.set(false);

            if let Some(data_transfer) = ev.data_transfer() {
                if let Some(files) = data_transfer.files() {
                    process_files(files);
                }
            }
        }
    };

    // Handle file input change
    let on_file_select = {
        let process_files = process_files.clone();
        move |ev: leptos::ev::Event| {
            let target = ev.target().unwrap();
            let input: HtmlInputElement = target.unchecked_into();
            if let Some(files) = input.files() {
                process_files(files);
            }
        }
    };

    // Browse button click
    let on_browse_click = move |_| {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    // Remove file from queue
    let remove_file = move |id: u32| {
        set_uploads.update(|u| u.retain(|item| item.id != id));
    };

    // Start upload - use store_value to allow multiple calls
    let bucket_for_upload = store_value(bucket_clone.clone());
    let prefix_for_upload = store_value(prefix_clone.clone());
    let on_upload_complete_stored = store_value(on_upload_complete.clone());

    let start_upload = move |_: web_sys::MouseEvent| {
        let uploads_list = uploads.get();
        if uploads_list.is_empty() || is_uploading.get() {
            return;
        }

        set_is_uploading.set(true);

        let bucket = bucket_for_upload.get_value();
        let prefix = prefix_for_upload.get_value();
        let on_complete = on_upload_complete_stored.get_value();

        spawn_local(async move {
            // Get file input and iterate through files
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            if let Some(input) = document.get_element_by_id("file-upload-input") {
                let input: HtmlInputElement = input.unchecked_into();
                if let Some(files) = input.files() {
                    for i in 0..files.length() {
                        if let Some(file) = files.get(i) {
                            let file_name = file.name();
                            let key = if prefix.is_empty() {
                                file_name.clone()
                            } else {
                                format!("{}{}", prefix, file_name)
                            };

                            // Update status to uploading
                            set_uploads.update(|u| {
                                if let Some(item) = u.iter_mut().find(|item| item.name == file_name) {
                                    item.status = UploadStatus::Uploading;
                                    item.progress = 0.0;
                                }
                            });

                            // Perform upload
                            match crate::api::upload_object(&bucket, &key, file.clone()).await {
                                Ok(_) => {
                                    set_uploads.update(|u| {
                                        if let Some(item) = u.iter_mut().find(|item| item.name == file_name) {
                                            item.status = UploadStatus::Complete;
                                            item.progress = 100.0;
                                        }
                                    });
                                }
                                Err(e) => {
                                    set_uploads.update(|u| {
                                        if let Some(item) = u.iter_mut().find(|item| item.name == file_name) {
                                            item.status = UploadStatus::Error;
                                            item.error = Some(e.message);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            }

            set_is_uploading.set(false);
            on_complete.call(());
        });
    };

    // Check if all uploads complete
    let all_complete = move || {
        let list = uploads.get();
        !list.is_empty() && list.iter().all(|u| u.status == UploadStatus::Complete)
    };

    let has_pending = move || {
        uploads.get().iter().any(|u| u.status == UploadStatus::Pending)
    };

    view! {
        // Modal backdrop
        <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4"
             on:click=move |_| on_close.call(())>

            // Modal content
            <div class="bg-gray-800 rounded-xl border border-gray-700 w-full max-w-2xl max-h-[90vh] overflow-hidden"
                 on:click=|ev| ev.stop_propagation()>

                // Header
                <div class="px-6 py-4 border-b border-gray-700 flex items-center justify-between">
                    <h2 class="text-xl font-semibold text-white">"Upload Files"</h2>
                    <button class="text-gray-400 hover:text-white transition-colors"
                            on:click=move |_| on_close.call(())>
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                // Content
                <div class="p-6 space-y-4 overflow-y-auto max-h-[60vh]">
                    // Destination info
                    <div class="text-sm text-gray-400">
                        "Uploading to: "
                        <span class="text-white font-medium">{&bucket}</span>
                        {if !prefix.is_empty() {
                            view! { <span class="text-gray-500">"/" {&prefix}</span> }.into_view()
                        } else {
                            view! {}.into_view()
                        }}
                    </div>

                    // Drop zone
                    <div
                        class=move || format!(
                            "border-2 border-dashed rounded-xl p-8 text-center transition-all {}",
                            if is_dragging.get() {
                                "border-blue-500 bg-blue-500/10"
                            } else {
                                "border-gray-600 hover:border-gray-500"
                            }
                        )
                        on:dragenter=on_dragenter
                        on:dragleave=on_dragleave
                        on:dragover=on_dragover
                        on:drop=on_drop
                    >
                        <input
                            type="file"
                            multiple=true
                            id="file-upload-input"
                            node_ref=file_input_ref
                            class="hidden"
                            on:change=on_file_select
                        />

                        <svg class="w-12 h-12 mx-auto text-gray-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
                        </svg>

                        <p class="text-gray-300 mb-2">"Drag and drop files here"</p>
                        <p class="text-gray-500 text-sm mb-4">"or"</p>

                        <button
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
                            on:click=on_browse_click
                        >
                            "Browse Files"
                        </button>
                    </div>

                    // Upload queue
                    {move || {
                        let list = uploads.get();
                        if list.is_empty() {
                            view! {}.into_view()
                        } else {
                            view! {
                                <div class="space-y-2">
                                    <h3 class="text-sm font-medium text-gray-400">"Files to upload"</h3>
                                    <div class="space-y-2 max-h-48 overflow-y-auto">
                                        {list.into_iter().map(|item| {
                                            let id = item.id;
                                            let can_remove = item.status == UploadStatus::Pending;
                                            view! {
                                                <UploadItemRow
                                                    item=item
                                                    on_remove=Callback::new(move |_| remove_file(id))
                                                    can_remove=can_remove
                                                />
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            }.into_view()
                        }
                    }}
                </div>

                // Footer
                <div class="px-6 py-4 border-t border-gray-700 flex items-center justify-end space-x-3">
                    <button
                        class="px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
                        on:click=move |_| on_close.call(())
                    >
                        {move || if all_complete() { "Close" } else { "Cancel" }}
                    </button>

                    <Show when=move || !all_complete() && has_pending()>
                        <button
                            class=move || format!(
                                "px-4 py-2 rounded-lg transition-colors {}",
                                if is_uploading.get() {
                                    "bg-blue-600/50 text-gray-300 cursor-not-allowed"
                                } else {
                                    "bg-blue-600 hover:bg-blue-700 text-white"
                                }
                            )
                            disabled=is_uploading
                            on:click=start_upload
                        >
                            {move || if is_uploading.get() {
                                view! {
                                    <span class="flex items-center">
                                        <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                        </svg>
                                        "Uploading..."
                                    </span>
                                }.into_view()
                            } else {
                                view! {
                                    <span class="flex items-center">
                                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                                        </svg>
                                        "Upload"
                                    </span>
                                }.into_view()
                            }}
                        </button>
                    </Show>
                </div>
            </div>
        </div>
    }
}

#[component]
fn UploadItemRow(
    item: UploadItem,
    #[prop(into)] on_remove: Callback<()>,
    can_remove: bool,
) -> impl IntoView {
    let status_icon = match item.status {
        UploadStatus::Pending => view! {
            <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        }.into_view(),
        UploadStatus::Uploading => view! {
            <svg class="w-5 h-5 text-blue-400 animate-spin" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
        }.into_view(),
        UploadStatus::Complete => view! {
            <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
        }.into_view(),
        UploadStatus::Error => view! {
            <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
        }.into_view(),
    };

    view! {
        <div class="bg-gray-750 rounded-lg p-3">
            <div class="flex items-center space-x-3">
                {status_icon}

                <div class="flex-1 min-w-0">
                    <p class="text-sm text-white truncate">{&item.name}</p>
                    <p class="text-xs text-gray-500">{format_size(item.size)}</p>
                </div>

                {if item.status == UploadStatus::Uploading {
                    view! {
                        <div class="text-xs text-blue-400">{format!("{:.0}%", item.progress)}</div>
                    }.into_view()
                } else if let Some(error) = &item.error {
                    view! {
                        <div class="text-xs text-red-400 truncate max-w-32" title=error.clone()>{error}</div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}

                {if can_remove {
                    view! {
                        <button
                            class="p-1 text-gray-400 hover:text-red-400 transition-colors"
                            on:click=move |_| on_remove.call(())
                        >
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
            </div>

            // Progress bar for uploading items
            {if item.status == UploadStatus::Uploading {
                view! {
                    <div class="mt-2 h-1 bg-gray-700 rounded-full overflow-hidden">
                        <div
                            class="h-full bg-blue-500 transition-all duration-300"
                            style=format!("width: {}%", item.progress)
                        ></div>
                    </div>
                }.into_view()
            } else {
                view! {}.into_view()
            }}
        </div>
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
