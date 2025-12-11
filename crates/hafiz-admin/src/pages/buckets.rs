//! Buckets management page

use leptos::*;
use leptos_router::use_params_map;
use crate::api::{self, BucketInfo};
use crate::components::{Button, ButtonVariant, Modal};

#[component]
pub fn BucketsPage() -> impl IntoView {
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_delete_modal, set_show_delete_modal) = create_signal(false);
    let (bucket_to_delete, set_bucket_to_delete) = create_signal(Option::<String>::None);
    let (new_bucket_name, set_new_bucket_name) = create_signal(String::new());
    let (creating, set_creating) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    // Fetch buckets
    let buckets = create_resource(|| (), |_| async move { api::list_buckets().await });

    let on_create = move |_| {
        set_creating.set(true);
        set_error.set(None);

        let name = new_bucket_name.get();
        spawn_local(async move {
            match api::create_bucket(&name).await {
                Ok(_) => {
                    set_show_create_modal.set(false);
                    set_new_bucket_name.set(String::new());
                    buckets.refetch();
                }
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                }
            }
            set_creating.set(false);
        });
    };

    let on_delete_confirm = move |_| {
        if let Some(name) = bucket_to_delete.get() {
            spawn_local(async move {
                if api::delete_bucket(&name).await.is_ok() {
                    buckets.refetch();
                }
                set_show_delete_modal.set(false);
                set_bucket_to_delete.set(None);
            });
        }
    };

    view! {
        <div class="space-y-6">
            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white">"Buckets"</h1>
                    <p class="text-gray-400 mt-1">"Manage your storage buckets"</p>
                </div>
                <Button on_click=Callback::new(move |_| set_show_create_modal.set(true))>
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                    </svg>
                    "Create Bucket"
                </Button>
            </div>

            // Buckets grid
            <Suspense fallback=move || view! { <BucketsGridSkeleton /> }>
                {move || buckets.get().map(|result| match result {
                    Ok(list) => {
                        if list.is_empty() {
                            view! {
                                <div class="bg-gray-800 rounded-xl border border-gray-700 p-12 text-center">
                                    <svg class="w-16 h-16 mx-auto text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                            d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                                    </svg>
                                    <h3 class="text-xl font-semibold text-white mb-2">"No buckets yet"</h3>
                                    <p class="text-gray-400 mb-6">"Create your first bucket to start storing objects"</p>
                                    <Button on_click=Callback::new(move |_| set_show_create_modal.set(true))>
                                        "Create Bucket"
                                    </Button>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    {list.into_iter().map(|bucket| {
                                        let name = bucket.name.clone();
                                        let name_for_delete = bucket.name.clone();
                                        view! {
                                            <BucketCard
                                                bucket=bucket
                                                on_delete=Callback::new(move |_| {
                                                    set_bucket_to_delete.set(Some(name_for_delete.clone()));
                                                    set_show_delete_modal.set(true);
                                                })
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_view()
                        }
                    }
                    Err(e) => view! {
                        <div class="bg-red-900/20 border border-red-500 rounded-xl p-6 text-center">
                            <p class="text-red-400">"Failed to load buckets: " {e.to_string()}</p>
                        </div>
                    }.into_view()
                })}
            </Suspense>

            // Create bucket modal
            <Modal
                title="Create Bucket"
                show=show_create_modal.into()
                on_close=Callback::new(move |_| set_show_create_modal.set(false))
            >
                <div class="space-y-4">
                    {move || error.get().map(|e| view! {
                        <div class="bg-red-900/50 border border-red-500 text-red-200 px-4 py-3 rounded">
                            {e}
                        </div>
                    })}

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">
                            "Bucket Name"
                        </label>
                        <input
                            type="text"
                            class="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg
                                   text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                            placeholder="my-bucket"
                            prop:value=move || new_bucket_name.get()
                            on:input=move |ev| set_new_bucket_name.set(event_target_value(&ev))
                        />
                        <p class="text-sm text-gray-400 mt-2">
                            "Use lowercase letters, numbers, and hyphens only"
                        </p>
                    </div>

                    <div class="flex justify-end space-x-3 pt-4">
                        <Button
                            variant=ButtonVariant::Secondary
                            on_click=Callback::new(move |_| set_show_create_modal.set(false))
                        >
                            "Cancel"
                        </Button>
                        <Button
                            loading=Some(creating.into())
                            on_click=Callback::new(on_create)
                        >
                            "Create"
                        </Button>
                    </div>
                </div>
            </Modal>

            // Delete confirmation modal
            <Modal
                title="Delete Bucket"
                show=show_delete_modal.into()
                on_close=Callback::new(move |_| set_show_delete_modal.set(false))
            >
                <div class="space-y-4">
                    <p class="text-gray-300">
                        "Are you sure you want to delete bucket "
                        <span class="font-semibold text-white">
                            {move || bucket_to_delete.get().unwrap_or_default()}
                        </span>
                        "? This action cannot be undone."
                    </p>
                    <div class="flex justify-end space-x-3 pt-4">
                        <Button
                            variant=ButtonVariant::Secondary
                            on_click=Callback::new(move |_| set_show_delete_modal.set(false))
                        >
                            "Cancel"
                        </Button>
                        <Button
                            variant=ButtonVariant::Danger
                            on_click=Callback::new(on_delete_confirm)
                        >
                            "Delete"
                        </Button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}

#[component]
fn BucketCard(bucket: BucketInfo, on_delete: Callback<()>) -> impl IntoView {
    let name = bucket.name.clone();
    let name_for_link = bucket.name.clone();

    view! {
        <div class="bg-gray-800 rounded-xl border border-gray-700 p-6 hover:border-gray-600 transition-colors">
            <div class="flex items-start justify-between mb-4">
                <div class="flex items-center space-x-3">
                    <div class="p-2 bg-blue-600/20 rounded-lg">
                        <svg class="w-6 h-6 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                        </svg>
                    </div>
                    <div>
                        <a
                            href=format!("/buckets/{}", name_for_link)
                            class="text-lg font-semibold text-white hover:text-blue-400 transition-colors"
                        >
                            {&bucket.name}
                        </a>
                    </div>
                </div>
                <button
                    class="p-2 text-gray-400 hover:text-red-400 transition-colors"
                    on:click=move |_| on_delete.call(())
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                </button>
            </div>

            <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                    <p class="text-gray-400">"Objects"</p>
                    <p class="text-white font-medium">{bucket.object_count}</p>
                </div>
                <div>
                    <p class="text-gray-400">"Size"</p>
                    <p class="text-white font-medium">{format_bytes(bucket.size)}</p>
                </div>
            </div>

            <div class="mt-4 pt-4 border-t border-gray-700 flex items-center justify-between">
                <span class="text-sm text-gray-400">
                    "Created " {format_date(&bucket.created_at)}
                </span>
                <div class="flex items-center space-x-2">
                    {bucket.versioning_enabled.then(|| view! {
                        <span class="px-2 py-1 text-xs bg-green-600/20 text-green-400 rounded">
                            "Versioning"
                        </span>
                    })}
                    {bucket.encryption_enabled.then(|| view! {
                        <span class="px-2 py-1 text-xs bg-purple-600/20 text-purple-400 rounded">
                            "Encrypted"
                        </span>
                    })}
                </div>
            </div>
        </div>
    }
}

#[component]
fn BucketsGridSkeleton() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 animate-pulse">
            {(0..6).map(|_| view! {
                <div class="bg-gray-800 rounded-xl border border-gray-700 p-6">
                    <div class="flex items-center space-x-3 mb-4">
                        <div class="w-10 h-10 bg-gray-700 rounded-lg"></div>
                        <div class="h-6 w-32 bg-gray-700 rounded"></div>
                    </div>
                    <div class="space-y-2">
                        <div class="h-4 w-20 bg-gray-700 rounded"></div>
                        <div class="h-4 w-24 bg-gray-700 rounded"></div>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}

/// Bucket detail page
#[component]
pub fn BucketDetailPage() -> impl IntoView {
    let params = use_params_map();
    let bucket_name = move || params.get().get("name").cloned().unwrap_or_default();

    let bucket = create_resource(bucket_name, |name| async move {
        api::get_bucket(&name).await
    });

    view! {
        <div class="space-y-6">
            // Breadcrumb
            <nav class="flex items-center space-x-2 text-sm">
                <a href="/buckets" class="text-gray-400 hover:text-white">"Buckets"</a>
                <span class="text-gray-600">"/"</span>
                <span class="text-white">{bucket_name}</span>
            </nav>

            <Suspense fallback=move || view! { <BucketDetailSkeleton /> }>
                {move || bucket.get().map(|result| match result {
                    Ok(info) => view! {
                        <div class="space-y-6">
                            // Header
                            <div class="flex items-center justify-between">
                                <div class="flex items-center space-x-4">
                                    <div class="p-3 bg-blue-600/20 rounded-xl">
                                        <svg class="w-8 h-8 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h1 class="text-2xl font-bold text-white">{&info.name}</h1>
                                        <p class="text-gray-400">"Created " {format_date(&info.created_at)}</p>
                                    </div>
                                </div>
                                <div class="flex items-center space-x-3">
                                    <Button variant=ButtonVariant::Secondary>
                                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0..." />
                                        </svg>
                                        "Settings"
                                    </Button>
                                    <Button>
                                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                                        </svg>
                                        "Upload"
                                    </Button>
                                </div>
                            </div>

                            // Stats
                            <div class="grid grid-cols-4 gap-4">
                                <div class="bg-gray-800 rounded-xl p-4 border border-gray-700">
                                    <p class="text-sm text-gray-400">"Objects"</p>
                                    <p class="text-2xl font-bold text-white">{info.object_count}</p>
                                </div>
                                <div class="bg-gray-800 rounded-xl p-4 border border-gray-700">
                                    <p class="text-sm text-gray-400">"Total Size"</p>
                                    <p class="text-2xl font-bold text-white">{format_bytes(info.size)}</p>
                                </div>
                                <div class="bg-gray-800 rounded-xl p-4 border border-gray-700">
                                    <p class="text-sm text-gray-400">"Versioning"</p>
                                    <p class="text-2xl font-bold text-white">
                                        {if info.versioning_enabled { "Enabled" } else { "Disabled" }}
                                    </p>
                                </div>
                                <div class="bg-gray-800 rounded-xl p-4 border border-gray-700">
                                    <p class="text-sm text-gray-400">"Encryption"</p>
                                    <p class="text-2xl font-bold text-white">
                                        {if info.encryption_enabled { "AES-256" } else { "None" }}
                                    </p>
                                </div>
                            </div>

                            // Objects browser link
                            <div class="bg-gray-800 rounded-xl border border-gray-700 p-6">
                                <a
                                    href=format!("/buckets/{}/objects/", info.name)
                                    class="flex items-center justify-between group"
                                >
                                    <div>
                                        <h3 class="text-lg font-semibold text-white group-hover:text-blue-400 transition-colors">
                                            "Browse Objects"
                                        </h3>
                                        <p class="text-gray-400">"View and manage objects in this bucket"</p>
                                    </div>
                                    <svg class="w-6 h-6 text-gray-400 group-hover:text-blue-400 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                                    </svg>
                                </a>
                            </div>
                        </div>
                    }.into_view(),
                    Err(e) => view! {
                        <div class="bg-red-900/20 border border-red-500 rounded-xl p-6 text-center">
                            <p class="text-red-400">"Bucket not found: " {e.to_string()}</p>
                        </div>
                    }.into_view()
                })}
            </Suspense>
        </div>
    }
}

#[component]
fn BucketDetailSkeleton() -> impl IntoView {
    view! {
        <div class="animate-pulse space-y-6">
            <div class="flex items-center space-x-4">
                <div class="w-14 h-14 bg-gray-700 rounded-xl"></div>
                <div>
                    <div class="h-8 w-48 bg-gray-700 rounded"></div>
                    <div class="h-4 w-32 bg-gray-700 rounded mt-2"></div>
                </div>
            </div>
            <div class="grid grid-cols-4 gap-4">
                {(0..4).map(|_| view! {
                    <div class="bg-gray-800 rounded-xl p-4 border border-gray-700">
                        <div class="h-4 w-16 bg-gray-700 rounded mb-2"></div>
                        <div class="h-8 w-24 bg-gray-700 rounded"></div>
                    </div>
                }).collect_view()}
            </div>
        </div>
    }
}

// Utility functions
fn format_bytes(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;
    const TB: i64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
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
    // Simple date formatting - in production use chrono
    if date.len() >= 10 {
        date[..10].to_string()
    } else {
        date.to_string()
    }
}
