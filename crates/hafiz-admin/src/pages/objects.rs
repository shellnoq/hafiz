//! Object browser page

use leptos::*;
use leptos_router::use_params_map;
use crate::api::{self, ObjectInfo};
use crate::components::{Button, ButtonVariant, FileUploadModal};

#[component]
pub fn ObjectsPage() -> impl IntoView {
    let params = use_params_map();
    let bucket_name = move || params.get().get("name").cloned().unwrap_or_default();
    let current_path = move || {
        params.get().get("path").cloned().unwrap_or_default()
    };

    let (show_upload_modal, set_show_upload_modal) = create_signal(false);
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);

    let objects = create_resource(
        move || (bucket_name(), current_path(), refresh_trigger.get()),
        |(bucket, prefix, _)| async move {
            api::list_objects(&bucket, &prefix).await
        },
    );

    let on_upload_complete = move || {
        set_refresh_trigger.update(|t| *t += 1);
    };

    view! {
        <div class="space-y-6">
            // Breadcrumb navigation
            <nav class="flex items-center space-x-2 text-sm flex-wrap">
                <a href="/buckets" class="text-gray-400 hover:text-white">"Buckets"</a>
                <span class="text-gray-600">"/"</span>
                <a href=format!("/buckets/{}", bucket_name()) class="text-gray-400 hover:text-white">
                    {bucket_name}
                </a>
                <span class="text-gray-600">"/"</span>
                {move || {
                    let path = current_path();
                    if path.is_empty() {
                        view! { <span class="text-white">"Objects"</span> }.into_view()
                    } else {
                        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
                        parts.into_iter().enumerate().map(|(i, part)| {
                            let href = format!("/buckets/{}/objects/{}", bucket_name(),
                                parts[..=i].join("/"));
                            view! {
                                <>
                                    <a href=href class="text-gray-400 hover:text-white">{part}</a>
                                    <span class="text-gray-600">"/"</span>
                                </>
                            }
                        }).collect_view()
                    }
                }}
            </nav>

            // Header with actions
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white">"Objects"</h1>
                    <p class="text-gray-400 mt-1">
                        {move || if current_path().is_empty() { "Root".to_string() } else { current_path() }}
                    </p>
                </div>
                <div class="flex items-center space-x-3">
                    <Button variant=ButtonVariant::Secondary>
                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
                        </svg>
                        "New Folder"
                    </Button>
                    <button
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors flex items-center"
                        on:click=move |_| set_show_upload_modal.set(true)
                    >
                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                        </svg>
                        "Upload"
                    </button>
                </div>
            </div>

            // Upload modal
            {move || if show_upload_modal.get() {
                let bucket = bucket_name();
                let prefix = current_path();
                view! {
                    <FileUploadModal
                        bucket=bucket
                        prefix=prefix
                        on_close=move |_| set_show_upload_modal.set(false)
                        on_upload_complete=move |_| {
                            on_upload_complete();
                            set_show_upload_modal.set(false);
                        }
                    />
                }.into_view()
            } else {
                view! {}.into_view()
            }}

            // Objects table
            <div class="bg-gray-800 rounded-xl border border-gray-700 overflow-hidden">
                <Suspense fallback=move || view! { <ObjectsTableSkeleton /> }>
                    {move || objects.get().map(|result| match result {
                        Ok(list) => {
                            if list.objects.is_empty() && list.common_prefixes.is_empty() {
                                view! {
                                    <EmptyState on_upload=move || set_show_upload_modal.set(true) />
                                }.into_view()
                            } else {
                                let bucket = bucket_name();
                                view! {
                                    <table class="w-full">
                                        <thead>
                                            <tr class="border-b border-gray-700 bg-gray-750">
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"Name"</th>
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"Size"</th>
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"Modified"</th>
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-700">
                                            // Folders (common prefixes)
                                            {list.common_prefixes.into_iter().map(|prefix| {
                                                let folder_name = prefix.trim_end_matches('/').rsplit('/').next().unwrap_or(&prefix);
                                                let href = format!("/buckets/{}/objects/{}", bucket_name(), prefix);
                                                view! {
                                                    <tr class="hover:bg-gray-750 transition-colors">
                                                        <td class="px-4 py-3">
                                                            <a href=href class="flex items-center space-x-3 text-white hover:text-blue-400">
                                                                <svg class="w-5 h-5 text-yellow-400" fill="currentColor" viewBox="0 0 24 24">
                                                                    <path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z" />
                                                                </svg>
                                                                <span>{folder_name}</span>
                                                            </a>
                                                        </td>
                                                        <td class="px-4 py-3 text-gray-400">"-"</td>
                                                        <td class="px-4 py-3 text-gray-400">"-"</td>
                                                        <td class="px-4 py-3"></td>
                                                    </tr>
                                                }
                                            }).collect_view()}

                                            // Objects
                                            {list.objects.into_iter().map(|obj| {
                                                let key = obj.key.clone();
                                                let file_name = key.rsplit('/').next().unwrap_or(&key).to_string();
                                                let bucket_clone = bucket.clone();
                                                view! {
                                                    <ObjectRow
                                                        bucket=bucket_clone
                                                        object=obj
                                                        name=file_name
                                                        on_delete=move || set_refresh_trigger.update(|t| *t += 1)
                                                    />
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_view()
                            }
                        }
                        Err(e) => view! {
                            <div class="p-6 text-center text-red-400">
                                "Failed to load objects: " {e.to_string()}
                            </div>
                        }.into_view()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn EmptyState(#[prop(into)] on_upload: Callback<()>) -> impl IntoView {
    view! {
        <div class="p-12 text-center">
            <svg class="w-16 h-16 mx-auto text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
            <h3 class="text-xl font-semibold text-white mb-2">"No objects"</h3>
            <p class="text-gray-400 mb-6">"This folder is empty. Upload your first file!"</p>
            <button
                class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors inline-flex items-center"
                on:click=move |_| on_upload.call(())
            >
                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                </svg>
                "Upload Files"
            </button>
        </div>
    }
}

#[component]
fn ObjectRow(
    bucket: String,
    object: ObjectInfo,
    name: String,
    #[prop(into)] on_delete: Callback<()>,
) -> impl IntoView {
    use wasm_bindgen_futures::spawn_local;

    let icon = get_file_icon(&name);
    let key = object.key.clone();
    let key_for_delete = key.clone();
    let key_for_download = key.clone();
    let bucket_for_delete = bucket.clone();
    let bucket_for_download = bucket.clone();
    let name_for_download = name.clone();

    let (is_deleting, set_is_deleting) = create_signal(false);
    let (is_downloading, set_is_downloading) = create_signal(false);

    let handle_delete = move |_| {
        let bucket = bucket_for_delete.clone();
        let key = key_for_delete.clone();
        let on_delete = on_delete.clone();

        // Confirm delete
        let window = web_sys::window().unwrap();
        if !window.confirm_with_message(&format!("Delete {}?", key)).unwrap_or(false) {
            return;
        }

        set_is_deleting.set(true);
        spawn_local(async move {
            match api::delete_object(&bucket, &key).await {
                Ok(_) => on_delete.call(()),
                Err(e) => {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message(&format!("Delete failed: {}", e.message))
                        .ok();
                }
            }
            set_is_deleting.set(false);
        });
    };

    let handle_download = move |_| {
        let bucket = bucket_for_download.clone();
        let key = key_for_download.clone();
        let filename = name_for_download.clone();

        set_is_downloading.set(true);
        spawn_local(async move {
            match api::download_object(&bucket, &key).await {
                Ok(data) => {
                    // Create blob and download
                    use wasm_bindgen::JsCast;
                    use js_sys::{Array, Uint8Array};

                    let uint8_array = Uint8Array::from(&data[..]);
                    let array = Array::new();
                    array.push(&uint8_array);

                    if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(&array) {
                        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();

                        let window = web_sys::window().unwrap();
                        let document = window.document().unwrap();
                        let a: web_sys::HtmlAnchorElement = document
                            .create_element("a")
                            .unwrap()
                            .unchecked_into();

                        a.set_href(&url);
                        a.set_download(&filename);
                        a.click();

                        web_sys::Url::revoke_object_url(&url).ok();
                    }
                }
                Err(e) => {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message(&format!("Download failed: {}", e.message))
                        .ok();
                }
            }
            set_is_downloading.set(false);
        });
    };

    view! {
        <tr class="hover:bg-gray-750 transition-colors">
            <td class="px-4 py-3">
                <div class="flex items-center space-x-3">
                    {icon}
                    <span class="text-white">{&name}</span>
                    {object.encryption.as_ref().map(|_| view! {
                        <span class="px-1.5 py-0.5 text-xs bg-purple-600/20 text-purple-400 rounded">
                            "🔒"
                        </span>
                    })}
                </div>
            </td>
            <td class="px-4 py-3 text-gray-400">{format_bytes(object.size)}</td>
            <td class="px-4 py-3 text-gray-400">{format_date(&object.last_modified)}</td>
            <td class="px-4 py-3">
                <div class="flex items-center space-x-2">
                    <button
                        class="p-2 text-gray-400 hover:text-white transition-colors disabled:opacity-50"
                        title="Download"
                        disabled=is_downloading
                        on:click=handle_download
                    >
                        {move || if is_downloading.get() {
                            view! {
                                <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                                </svg>
                            }.into_view()
                        } else {
                            view! {
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                        d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                </svg>
                            }.into_view()
                        }}
                    </button>
                    <button
                        class="p-2 text-gray-400 hover:text-red-400 transition-colors disabled:opacity-50"
                        title="Delete"
                        disabled=is_deleting
                        on:click=handle_delete
                    >
                        {move || if is_deleting.get() {
                            view! {
                                <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
                                </svg>
                            }.into_view()
                        } else {
                            view! {
                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                </svg>
                            }.into_view()
                        }}
                    </button>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn ObjectsTableSkeleton() -> impl IntoView {
    view! {
        <div class="animate-pulse">
            <div class="border-b border-gray-700 px-4 py-3">
                <div class="h-4 w-48 bg-gray-700 rounded"></div>
            </div>
            {(0..5).map(|_| view! {
                <div class="border-b border-gray-700 px-4 py-3 flex items-center space-x-4">
                    <div class="w-5 h-5 bg-gray-700 rounded"></div>
                    <div class="h-4 w-48 bg-gray-700 rounded"></div>
                    <div class="flex-1"></div>
                    <div class="h-4 w-16 bg-gray-700 rounded"></div>
                    <div class="h-4 w-24 bg-gray-700 rounded"></div>
                </div>
            }).collect_view()}
        </div>
    }
}

fn get_file_icon(name: &str) -> impl IntoView {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    let (color, icon_path) = match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" => ("text-pink-400", "M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"),
        "mp4" | "mov" | "avi" | "mkv" => ("text-red-400", "M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"),
        "mp3" | "wav" | "ogg" | "flac" => ("text-purple-400", "M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"),
        "pdf" => ("text-red-500", "M12 10v6m0 0l-3-3m3 3l3-3M3 17V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"),
        "zip" | "tar" | "gz" | "rar" | "7z" => ("text-yellow-400", "M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4"),
        "js" | "ts" | "jsx" | "tsx" => ("text-yellow-300", "M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"),
        "rs" | "py" | "go" | "java" | "c" | "cpp" | "h" => ("text-blue-400", "M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"),
        "json" | "yaml" | "yml" | "toml" | "xml" => ("text-green-400", "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"),
        "md" | "txt" | "log" => ("text-gray-400", "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"),
        _ => ("text-gray-400", "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"),
    };

    view! {
        <svg class=format!("w-5 h-5 {}", color) fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d=icon_path />
        </svg>
    }
}

fn format_bytes(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

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

fn format_date(date: &str) -> String {
    if date.len() >= 10 {
        date[..10].to_string()
    } else {
        date.to_string()
    }
}
