//! Settings page

use leptos::*;
use crate::api;
use crate::components::{Button, ButtonVariant};

#[component]
pub fn SettingsPage() -> impl IntoView {
    let server_info = create_resource(|| (), |_| async move { api::get_server_info().await });
    let health_status = create_resource(|| (), |_| async move { api::health_check().await });

    view! {
        <div class="space-y-6">
            // Page header
            <div>
                <h1 class="text-2xl font-bold text-white">"Settings"</h1>
                <p class="text-gray-400 mt-1">"Configure your Hafiz instance"</p>
            </div>

            // Settings sections
            <div class="space-y-6">
                // Server Information
                <SettingsCard title="Server Information" description="Current server configuration and status">
                    <Suspense fallback=move || view! { <SettingsSkeleton /> }>
                        {move || server_info.get().map(|result| match result {
                            Ok(info) => view! {
                                <div class="grid grid-cols-2 gap-4">
                                    <SettingItem label="Version" value=info.version />
                                    <SettingItem label="S3 API Endpoint" value=info.s3_endpoint />
                                    <SettingItem label="Admin API Endpoint" value=info.admin_endpoint />
                                    <SettingItem label="Storage Backend" value=info.storage_backend />
                                    <SettingItem label="Database" value=info.database_type />
                                    <SettingItem label="Uptime" value=info.uptime />
                                </div>
                            }.into_view(),
                            Err(_) => view! { <p class="text-red-400">"Failed to load server info"</p> }.into_view()
                        })}
                    </Suspense>
                </SettingsCard>

                // Health Status
                <SettingsCard title="System Health" description="Current health status of system components">
                    <Suspense fallback=move || view! { <SettingsSkeleton /> }>
                        {move || health_status.get().map(|result| match result {
                            Ok(health) => {
                                let status_class = if health.status == "healthy" {
                                    "text-green-400 font-medium"
                                } else {
                                    "text-yellow-400 font-medium"
                                };
                                let status_text = health.status.clone();
                                view! {
                                    <div class="space-y-3">
                                        <div class="flex items-center justify-between p-3 bg-gray-750 rounded-lg">
                                            <span class="text-gray-300">"Overall Status"</span>
                                            <span class=status_class>
                                                {status_text}
                                            </span>
                                        </div>
                                        <div class="flex items-center justify-between p-3 bg-gray-750 rounded-lg">
                                            <span class="text-gray-300">"Storage"</span>
                                            <span class=if health.storage_ok { "text-green-400" } else { "text-red-400" }>
                                                {if health.storage_ok { "✓ OK" } else { "✗ Error" }}
                                            </span>
                                        </div>
                                        <div class="flex items-center justify-between p-3 bg-gray-750 rounded-lg">
                                            <span class="text-gray-300">"Database"</span>
                                            <span class=if health.database_ok { "text-green-400" } else { "text-red-400" }>
                                                {if health.database_ok { "✓ OK" } else { "✗ Error" }}
                                            </span>
                                        </div>
                                    </div>
                                }
                            }.into_view(),
                            Err(_) => view! { <p class="text-red-400">"Failed to load health status"</p> }.into_view()
                        })}
                    </Suspense>
                </SettingsCard>

                // Security Settings
                <SettingsCard title="Security" description="Encryption and access control settings">
                    <div class="space-y-4">
                        <SettingToggle
                            label="Server-Side Encryption (SSE-S3)"
                            description="Enable encryption at rest with server-managed keys"
                            enabled=true
                        />
                        <SettingToggle
                            label="Customer-Provided Keys (SSE-C)"
                            description="Allow customers to provide their own encryption keys"
                            enabled=true
                        />
                        <SettingToggle
                            label="HTTPS Only"
                            description="Require HTTPS for all connections"
                            enabled=false
                        />
                    </div>
                </SettingsCard>

                // Storage Settings
                <SettingsCard title="Storage" description="Storage configuration and limits">
                    <div class="space-y-4">
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Max Object Size"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    value="5 TiB"
                                    disabled
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Data Directory"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    value="/data/hafiz"
                                    disabled
                                />
                            </div>
                        </div>
                    </div>
                </SettingsCard>

                // Lifecycle Settings
                <SettingsCard title="Lifecycle Worker" description="Automatic object expiration and cleanup">
                    <div class="space-y-4">
                        <SettingToggle
                            label="Enable Lifecycle Worker"
                            description="Automatically process lifecycle rules"
                            enabled=true
                        />
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Scan Interval"
                                </label>
                                <select class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                               text-white focus:outline-none focus:border-blue-500">
                                    <option value="3600">"1 hour"</option>
                                    <option value="21600">"6 hours"</option>
                                    <option value="86400">"24 hours"</option>
                                </select>
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Batch Size"
                                </label>
                                <input
                                    type="number"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    value="1000"
                                />
                            </div>
                        </div>
                    </div>
                </SettingsCard>

                // Danger Zone
                <SettingsCard title="Danger Zone" description="Irreversible actions" danger=true>
                    <div class="space-y-4">
                        <div class="flex items-center justify-between p-4 bg-red-900/20 border border-red-800 rounded-lg">
                            <div>
                                <p class="text-white font-medium">"Clear All Data"</p>
                                <p class="text-sm text-gray-400">"Delete all buckets, objects, and users"</p>
                            </div>
                            <Button variant=ButtonVariant::Danger>
                                "Clear Data"
                            </Button>
                        </div>
                    </div>
                </SettingsCard>
            </div>
        </div>
    }
}

#[component]
fn SettingsCard(
    title: &'static str,
    description: &'static str,
    children: Children,
    #[prop(optional)] danger: Option<bool>,
) -> impl IntoView {
    let border_class = if danger.unwrap_or(false) {
        "border-red-800"
    } else {
        "border-gray-700"
    };

    view! {
        <div class=format!("bg-gray-800 rounded-xl border {} p-6", border_class)>
            <div class="mb-4">
                <h2 class="text-lg font-semibold text-white">{title}</h2>
                <p class="text-sm text-gray-400">{description}</p>
            </div>
            {children()}
        </div>
    }
}

#[component]
fn SettingItem(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="p-3 bg-gray-750 rounded-lg">
            <p class="text-sm text-gray-400">{label}</p>
            <p class="text-white font-medium">{value}</p>
        </div>
    }
}

#[component]
fn SettingToggle(
    label: &'static str,
    description: &'static str,
    enabled: bool,
) -> impl IntoView {
    let (is_enabled, set_enabled) = create_signal(enabled);

    view! {
        <div class="flex items-center justify-between p-4 bg-gray-750 rounded-lg">
            <div>
                <p class="text-white font-medium">{label}</p>
                <p class="text-sm text-gray-400">{description}</p>
            </div>
            <button
                class=move || {
                    let base = "relative inline-flex h-6 w-11 items-center rounded-full transition-colors";
                    if is_enabled.get() {
                        format!("{} bg-blue-600", base)
                    } else {
                        format!("{} bg-gray-600", base)
                    }
                }
                on:click=move |_| set_enabled.update(|v| *v = !*v)
            >
                <span
                    class=move || {
                        let base = "inline-block h-4 w-4 transform rounded-full bg-white transition-transform";
                        if is_enabled.get() {
                            format!("{} translate-x-6", base)
                        } else {
                            format!("{} translate-x-1", base)
                        }
                    }
                />
            </button>
        </div>
    }
}

#[component]
fn SettingsSkeleton() -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 gap-4 animate-pulse">
            {(0..6).map(|_| view! {
                <div class="p-3 bg-gray-700 rounded-lg">
                    <div class="h-4 w-16 bg-gray-600 rounded mb-2"></div>
                    <div class="h-5 w-32 bg-gray-600 rounded"></div>
                </div>
            }).collect_view()}
        </div>
    }
}
