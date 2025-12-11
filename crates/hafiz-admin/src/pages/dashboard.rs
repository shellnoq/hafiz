//! Dashboard page with overview statistics

use leptos::*;
use crate::api::{self, DashboardStats};

#[component]
pub fn DashboardPage() -> impl IntoView {
    let stats = create_resource(|| (), |_| async move { api::get_dashboard_stats().await });

    view! {
        <div class="space-y-6">
            // Page header
            <div>
                <h1 class="text-2xl font-bold text-white">"Dashboard"</h1>
                <p class="text-gray-400 mt-1">"Overview of your storage system"</p>
            </div>

            // Stats cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <Suspense fallback=move || view! { <StatsCardSkeleton /> }>
                    {move || stats.get().map(|result| match result {
                        Ok(s) => view! {
                            <>
                                <StatCardItem
                                    title="Total Buckets"
                                    value=s.total_buckets.to_string()
                                    icon=IconBucket
                                    color="blue"
                                />
                                <StatCardItem
                                    title="Total Objects"
                                    value=format_number(s.total_objects)
                                    icon=IconFile
                                    color="green"
                                />
                                <StatCardItem
                                    title="Total Storage"
                                    value=format_bytes(s.total_size)
                                    icon=IconStorage
                                    color="purple"
                                />
                                <StatCardItem
                                    title="Total Users"
                                    value=s.total_users.to_string()
                                    icon=IconUsers
                                    color="orange"
                                />
                            </>
                        }.into_view(),
                        Err(_) => view! {
                            <div class="col-span-4 text-center text-red-400">
                                "Failed to load dashboard stats"
                            </div>
                        }.into_view()
                    })}
                </Suspense>
            </div>

            // Recent activity and quick actions
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                // Recent buckets
                <div class="bg-gray-800 rounded-xl border border-gray-700 p-6">
                    <h2 class="text-lg font-semibold text-white mb-4">"Recent Buckets"</h2>
                    <Suspense fallback=move || view! { <TableSkeleton rows=5 /> }>
                        {move || stats.get().map(|result| match result {
                            Ok(s) => view! {
                                <div class="space-y-3">
                                    {s.recent_buckets.into_iter().map(|bucket| view! {
                                        <a
                                            href=format!("/buckets/{}", bucket.name)
                                            class="flex items-center justify-between p-3 bg-gray-750 rounded-lg hover:bg-gray-700 transition-colors"
                                        >
                                            <div class="flex items-center space-x-3">
                                                <div class="p-2 bg-blue-600/20 rounded-lg">
                                                    <IconBucket />
                                                </div>
                                                <div>
                                                    <p class="text-white font-medium">{&bucket.name}</p>
                                                    <p class="text-sm text-gray-400">
                                                        {bucket.object_count} " objects"
                                                    </p>
                                                </div>
                                            </div>
                                            <span class="text-sm text-gray-400">
                                                {format_bytes(bucket.size)}
                                            </span>
                                        </a>
                                    }).collect_view()}
                                    {s.recent_buckets.is_empty().then(|| view! {
                                        <p class="text-gray-400 text-center py-4">"No buckets yet"</p>
                                    })}
                                </div>
                            }.into_view(),
                            Err(_) => view! { <p class="text-red-400">"Failed to load"</p> }.into_view()
                        })}
                    </Suspense>
                </div>

                // Quick actions
                <div class="bg-gray-800 rounded-xl border border-gray-700 p-6">
                    <h2 class="text-lg font-semibold text-white mb-4">"Quick Actions"</h2>
                    <div class="grid grid-cols-2 gap-4">
                        <QuickAction
                            href="/buckets?action=create"
                            icon=IconPlus
                            title="Create Bucket"
                            description="Add a new storage bucket"
                        />
                        <QuickAction
                            href="/users?action=create"
                            icon=IconUserPlus
                            title="Add User"
                            description="Create a new user account"
                        />
                        <QuickAction
                            href="/settings"
                            icon=IconSettings
                            title="Settings"
                            description="Configure your storage"
                        />
                        <QuickAction
                            href="/buckets"
                            icon=IconBucket
                            title="Browse Buckets"
                            description="View all buckets"
                        />
                    </div>
                </div>
            </div>

            // System info
            <div class="bg-gray-800 rounded-xl border border-gray-700 p-6">
                <h2 class="text-lg font-semibold text-white mb-4">"System Information"</h2>
                <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                    <SystemInfoItem label="Version" value="0.1.0" />
                    <SystemInfoItem label="S3 API" value="Compatible" />
                    <SystemInfoItem label="Encryption" value="AES-256" />
                    <SystemInfoItem label="Status" value="Healthy" status="green" />
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatCardItem(
    title: &'static str,
    value: String,
    icon: fn() -> impl IntoView,
    color: &'static str,
) -> impl IntoView {
    let bg_class = match color {
        "blue" => "bg-blue-600/20",
        "green" => "bg-green-600/20",
        "purple" => "bg-purple-600/20",
        "orange" => "bg-orange-600/20",
        _ => "bg-gray-600/20",
    };

    let icon_class = match color {
        "blue" => "text-blue-400",
        "green" => "text-green-400",
        "purple" => "text-purple-400",
        "orange" => "text-orange-400",
        _ => "text-gray-400",
    };

    view! {
        <div class="bg-gray-800 rounded-xl p-6 border border-gray-700">
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm font-medium text-gray-400">{title}</p>
                    <p class="text-3xl font-bold text-white mt-2">{value}</p>
                </div>
                <div class=format!("p-3 rounded-lg {}", bg_class)>
                    <div class=icon_class>
                        {icon()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn QuickAction(
    href: &'static str,
    icon: fn() -> impl IntoView,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="flex items-start space-x-3 p-4 bg-gray-750 rounded-lg hover:bg-gray-700 transition-colors"
        >
            <div class="p-2 bg-blue-600/20 rounded-lg text-blue-400">
                {icon()}
            </div>
            <div>
                <p class="text-white font-medium">{title}</p>
                <p class="text-sm text-gray-400">{description}</p>
            </div>
        </a>
    }
}

#[component]
fn SystemInfoItem(
    label: &'static str,
    value: &'static str,
    #[prop(optional)] status: Option<&'static str>,
) -> impl IntoView {
    let value_class = match status {
        Some("green") => "text-green-400",
        Some("red") => "text-red-400",
        Some("yellow") => "text-yellow-400",
        _ => "text-white",
    };

    view! {
        <div class="p-4 bg-gray-750 rounded-lg">
            <p class="text-sm text-gray-400">{label}</p>
            <p class=format!("font-medium {}", value_class)>{value}</p>
        </div>
    }
}

#[component]
fn StatsCardSkeleton() -> impl IntoView {
    view! {
        <>
            {(0..4).map(|_| view! {
                <div class="bg-gray-800 rounded-xl p-6 border border-gray-700 animate-pulse">
                    <div class="h-4 w-20 bg-gray-700 rounded mb-4"></div>
                    <div class="h-8 w-32 bg-gray-700 rounded"></div>
                </div>
            }).collect_view()}
        </>
    }
}

#[component]
fn TableSkeleton(rows: usize) -> impl IntoView {
    view! {
        <div class="space-y-3 animate-pulse">
            {(0..rows).map(|_| view! {
                <div class="h-16 bg-gray-700 rounded-lg"></div>
            }).collect_view()}
        </div>
    }
}

// Icons
#[component]
fn IconBucket() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
        </svg>
    }
}

#[component]
fn IconFile() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
        </svg>
    }
}

#[component]
fn IconStorage() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
        </svg>
    }
}

#[component]
fn IconUsers() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
    }
}

#[component]
fn IconPlus() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
    }
}

#[component]
fn IconUserPlus() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z" />
        </svg>
    }
}

#[component]
fn IconSettings() -> impl IntoView {
    view! {
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
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

fn format_number(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
