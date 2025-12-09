//! Cluster management page
//!
//! Displays cluster status, nodes, replication rules, and statistics.

use leptos::*;
use crate::api::{self, ClusterStatus, NodeInfo, ReplicationRule, ReplicationStats, NodesList, ReplicationRulesList, ClusterHealth};
use crate::components::{Button, Modal, StatCard};

/// Cluster page component
#[component]
pub fn ClusterPage() -> impl IntoView {
    // State
    let (cluster_status, set_cluster_status) = create_signal(None::<ClusterStatus>);
    let (nodes, set_nodes) = create_signal(Vec::<NodeInfo>::new());
    let (rules, set_rules) = create_signal(Vec::<ReplicationRule>::new());
    let (replication_stats, set_replication_stats) = create_signal(None::<ReplicationStats>);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(None::<String>);
    let (active_tab, set_active_tab) = create_signal("overview".to_string());

    // Modal state
    let (show_rule_modal, set_show_rule_modal) = create_signal(false);
    let (new_rule_bucket, set_new_rule_bucket) = create_signal(String::new());
    let (new_rule_prefix, set_new_rule_prefix) = create_signal(String::new());
    let (new_rule_mode, set_new_rule_mode) = create_signal("async".to_string());

    // Load cluster data
    let load_data = move || {
        set_loading.set(true);
        set_error.set(None);

        spawn_local(async move {
            // Load cluster status
            match api::get_cluster_status().await {
                Ok(status) => set_cluster_status.set(Some(status)),
                Err(e) => {
                    // Cluster might be disabled
                    if e.message.contains("not enabled") {
                        set_cluster_status.set(Some(ClusterStatus {
                            enabled: false,
                            ..Default::default()
                        }));
                    } else {
                        set_error.set(Some(e.message));
                    }
                }
            }

            // Load nodes
            match api::list_cluster_nodes().await {
                Ok(list) => set_nodes.set(list.nodes),
                Err(_) => {} // Ignore if cluster disabled
            }

            // Load replication rules
            match api::list_replication_rules().await {
                Ok(list) => set_rules.set(list.rules),
                Err(_) => {}
            }

            // Load replication stats
            match api::get_replication_stats().await {
                Ok(stats) => set_replication_stats.set(Some(stats)),
                Err(_) => {}
            }

            set_loading.set(false);
        });
    };

    // Initial load
    load_data();

    // Create replication rule
    let create_rule = move |_| {
        let bucket = new_rule_bucket.get();
        let prefix = new_rule_prefix.get();
        let mode = new_rule_mode.get();

        if bucket.is_empty() {
            return;
        }

        spawn_local(async move {
            let request = api::CreateReplicationRuleRequest {
                source_bucket: bucket,
                destination_bucket: None,
                target_nodes: None,
                prefix_filter: if prefix.is_empty() { None } else { Some(prefix) },
                mode: Some(mode),
                replicate_deletes: Some(true),
            };

            match api::create_replication_rule(&request).await {
                Ok(_) => {
                    set_show_rule_modal.set(false);
                    set_new_rule_bucket.set(String::new());
                    set_new_rule_prefix.set(String::new());
                    // Reload rules
                    if let Ok(list) = api::list_replication_rules().await {
                        set_rules.set(list.rules);
                    }
                }
                Err(e) => set_error.set(Some(e.message)),
            }
        });
    };

    // Delete replication rule
    let delete_rule = move |rule_id: String| {
        spawn_local(async move {
            match api::delete_replication_rule(&rule_id).await {
                Ok(_) => {
                    if let Ok(list) = api::list_replication_rules().await {
                        set_rules.set(list.rules);
                    }
                }
                Err(e) => set_error.set(Some(e.message)),
            }
        });
    };

    view! {
        <div class="p-6">
            // Header
            <div class="flex justify-between items-center mb-6">
                <div>
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">"Cluster Management"</h1>
                    <p class="text-gray-500 dark:text-gray-400">"Monitor and manage cluster nodes and replication"</p>
                </div>
                <Button on_click=move |_| load_data() variant="secondary">
                    <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                    </svg>
                    "Refresh"
                </Button>
            </div>

            // Error message
            {move || error.get().map(|e| view! {
                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                    {e}
                </div>
            })}

            // Loading state
            <Show when=move || loading.get()>
                <div class="flex justify-center items-center h-64">
                    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
                </div>
            </Show>

            // Cluster disabled message
            <Show when=move || {
                !loading.get() && cluster_status.get().map(|s| !s.enabled).unwrap_or(true)
            }>
                <div class="bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg p-6 text-center">
                    <svg class="w-12 h-12 mx-auto text-yellow-500 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                    </svg>
                    <h3 class="text-lg font-semibold text-yellow-800 dark:text-yellow-200 mb-2">"Cluster Mode Disabled"</h3>
                    <p class="text-yellow-700 dark:text-yellow-300 mb-4">
                        "Cluster features are not enabled. Configure seed nodes in your config to enable clustering."
                    </p>
                    <pre class="bg-yellow-100 dark:bg-yellow-900/40 p-4 rounded text-left text-sm overflow-x-auto">
                        {"[cluster]\nenabled = true\nname = \"my-cluster\"\nseed_nodes = [\"http://node1:9001\"]"}
                    </pre>
                </div>
            </Show>

            // Main content when cluster is enabled
            <Show when=move || {
                !loading.get() && cluster_status.get().map(|s| s.enabled).unwrap_or(false)
            }>
                // Tabs
                <div class="border-b border-gray-200 dark:border-gray-700 mb-6">
                    <nav class="-mb-px flex space-x-8">
                        <TabButton
                            label="Overview"
                            tab="overview"
                            active_tab=active_tab
                            set_active_tab=set_active_tab
                        />
                        <TabButton
                            label="Nodes"
                            tab="nodes"
                            active_tab=active_tab
                            set_active_tab=set_active_tab
                        />
                        <TabButton
                            label="Replication"
                            tab="replication"
                            active_tab=active_tab
                            set_active_tab=set_active_tab
                        />
                    </nav>
                </div>

                // Tab content
                <div>
                    // Overview Tab
                    <Show when=move || active_tab.get() == "overview">
                        <ClusterOverview
                            status=cluster_status
                            stats=replication_stats
                            nodes=nodes
                        />
                    </Show>

                    // Nodes Tab
                    <Show when=move || active_tab.get() == "nodes">
                        <NodesTable nodes=nodes />
                    </Show>

                    // Replication Tab
                    <Show when=move || active_tab.get() == "replication">
                        <ReplicationPanel
                            rules=rules
                            stats=replication_stats
                            on_create=move |_| set_show_rule_modal.set(true)
                            on_delete=delete_rule
                        />
                    </Show>
                </div>
            </Show>

            // Create Rule Modal
            <Modal
                show=show_rule_modal
                on_close=move |_| set_show_rule_modal.set(false)
                title="Create Replication Rule"
            >
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Source Bucket"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                            placeholder="my-bucket"
                            prop:value=move || new_rule_bucket.get()
                            on:input=move |ev| set_new_rule_bucket.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Prefix Filter (optional)"
                        </label>
                        <input
                            type="text"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                            placeholder="logs/"
                            prop:value=move || new_rule_prefix.get()
                            on:input=move |ev| set_new_rule_prefix.set(event_target_value(&ev))
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            "Replication Mode"
                        </label>
                        <select
                            class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                            on:change=move |ev| set_new_rule_mode.set(event_target_value(&ev))
                        >
                            <option value="async" selected>"Async (Eventual Consistency)"</option>
                            <option value="sync">"Sync (Strong Consistency)"</option>
                        </select>
                    </div>
                    <div class="flex justify-end space-x-3 pt-4">
                        <Button variant="secondary" on_click=move |_| set_show_rule_modal.set(false)>
                            "Cancel"
                        </Button>
                        <Button variant="primary" on_click=create_rule>
                            "Create Rule"
                        </Button>
                    </div>
                </div>
            </Modal>
        </div>
    }
}

/// Tab button component
#[component]
fn TabButton(
    label: &'static str,
    tab: &'static str,
    active_tab: ReadSignal<String>,
    set_active_tab: WriteSignal<String>,
) -> impl IntoView {
    let is_active = move || active_tab.get() == tab;

    view! {
        <button
            class=move || if is_active() {
                "border-b-2 border-indigo-500 py-4 px-1 text-sm font-medium text-indigo-600 dark:text-indigo-400"
            } else {
                "border-b-2 border-transparent py-4 px-1 text-sm font-medium text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300"
            }
            on:click=move |_| set_active_tab.set(tab.to_string())
        >
            {label}
        </button>
    }
}

/// Cluster overview component
#[component]
fn ClusterOverview(
    status: ReadSignal<Option<ClusterStatus>>,
    stats: ReadSignal<Option<ReplicationStats>>,
    nodes: ReadSignal<Vec<NodeInfo>>,
) -> impl IntoView {
    view! {
        <div class="space-y-6">
            // Cluster info card
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Cluster Information"</h3>
                {move || status.get().map(|s| view! {
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                        <div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">"Cluster Name"</p>
                            <p class="font-medium text-gray-900 dark:text-white">{&s.cluster_name}</p>
                        </div>
                        <div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">"Local Node"</p>
                            <p class="font-medium text-gray-900 dark:text-white">{&s.local_node.name}</p>
                        </div>
                        <div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">"Role"</p>
                            <span class=move || format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
                                if s.local_node.role == "primary" { "bg-green-100 text-green-800" } else { "bg-blue-100 text-blue-800" }
                            )>
                                {&s.local_node.role}
                            </span>
                        </div>
                        <div>
                            <p class="text-sm text-gray-500 dark:text-gray-400">"Status"</p>
                            <span class=move || format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
                                if s.local_node.status == "healthy" { "bg-green-100 text-green-800" } else { "bg-yellow-100 text-yellow-800" }
                            )>
                                {&s.local_node.status}
                            </span>
                        </div>
                    </div>
                })}
            </div>

            // Stats cards
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {move || status.get().map(|s| view! {
                    <StatCard
                        title="Total Nodes"
                        value=s.stats.total_nodes.to_string()
                        icon="server"
                        color="blue"
                    />
                    <StatCard
                        title="Healthy Nodes"
                        value=s.stats.healthy_nodes.to_string()
                        icon="check-circle"
                        color="green"
                    />
                })}
                {move || stats.get().map(|s| view! {
                    <StatCard
                        title="Pending Replications"
                        value=s.pending.to_string()
                        icon="clock"
                        color="yellow"
                    />
                    <StatCard
                        title="Failed Replications"
                        value=s.failed.to_string()
                        icon="x-circle"
                        color="red"
                    />
                })}
            </div>

            // Node status overview
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-4">"Node Status"</h3>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    {move || nodes.get().iter().take(6).map(|node| {
                        let status_color = if node.status == "healthy" { "green" } else { "yellow" };
                        view! {
                            <div class="flex items-center p-3 bg-gray-50 dark:bg-gray-700 rounded-lg">
                                <div class=format!("w-3 h-3 rounded-full bg-{}-500 mr-3", status_color)></div>
                                <div>
                                    <p class="font-medium text-gray-900 dark:text-white">{&node.name}</p>
                                    <p class="text-sm text-gray-500 dark:text-gray-400">{&node.role}</p>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </div>
    }
}

/// Nodes table component
#[component]
fn NodesTable(nodes: ReadSignal<Vec<NodeInfo>>) -> impl IntoView {
    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                <thead class="bg-gray-50 dark:bg-gray-700">
                    <tr>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Node"</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Endpoint"</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Role"</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Status"</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Last Heartbeat"</th>
                        <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">"Version"</th>
                    </tr>
                </thead>
                <tbody class="bg-white dark:bg-gray-800 divide-y divide-gray-200 dark:divide-gray-700">
                    {move || nodes.get().iter().map(|node| {
                        let status_class = if node.status == "healthy" {
                            "bg-green-100 text-green-800"
                        } else if node.status == "degraded" {
                            "bg-yellow-100 text-yellow-800"
                        } else {
                            "bg-red-100 text-red-800"
                        };
                        let role_class = if node.role == "primary" {
                            "bg-indigo-100 text-indigo-800"
                        } else {
                            "bg-gray-100 text-gray-800"
                        };

                        view! {
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <div class="flex items-center">
                                        <div class="flex-shrink-0">
                                            <svg class="h-8 w-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2"/>
                                            </svg>
                                        </div>
                                        <div class="ml-4">
                                            <div class="text-sm font-medium text-gray-900 dark:text-white">{&node.name}</div>
                                            <div class="text-sm text-gray-500 dark:text-gray-400">{&node.id}</div>
                                        </div>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                    {&node.endpoint}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}", role_class)>
                                        {&node.role}
                                    </span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap">
                                    <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}", status_class)>
                                        {&node.status}
                                    </span>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                    {&node.last_heartbeat}
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                                    {&node.version}
                                </td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>

            // Empty state
            <Show when=move || nodes.get().is_empty()>
                <div class="text-center py-12">
                    <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2"/>
                    </svg>
                    <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-white">"No nodes"</h3>
                    <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">"No cluster nodes found."</p>
                </div>
            </Show>
        </div>
    }
}

/// Replication panel component
#[component]
fn ReplicationPanel<F, G>(
    rules: ReadSignal<Vec<ReplicationRule>>,
    stats: ReadSignal<Option<ReplicationStats>>,
    on_create: F,
    on_delete: G,
) -> impl IntoView
where
    F: Fn(()) + 'static,
    G: Fn(String) + Clone + 'static,
{
    view! {
        <div class="space-y-6">
            // Replication stats
            {move || stats.get().map(|s| view! {
                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">"Events Processed"</p>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white">{s.events_processed}</p>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">"Successful"</p>
                        <p class="text-2xl font-bold text-green-600">{s.successful}</p>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">"Failed"</p>
                        <p class="text-2xl font-bold text-red-600">{s.failed}</p>
                    </div>
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
                        <p class="text-sm text-gray-500 dark:text-gray-400">"Bytes Replicated"</p>
                        <p class="text-2xl font-bold text-gray-900 dark:text-white">{format_bytes(s.bytes_replicated)}</p>
                    </div>
                </div>
            })}

            // Rules section
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow">
                <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
                    <h3 class="text-lg font-semibold text-gray-900 dark:text-white">"Replication Rules"</h3>
                    <Button variant="primary" on_click=move |_| on_create(())>
                        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
                        </svg>
                        "Create Rule"
                    </Button>
                </div>

                <div class="divide-y divide-gray-200 dark:divide-gray-700">
                    {move || rules.get().iter().map(|rule| {
                        let on_delete = on_delete.clone();
                        let rule_id = rule.id.clone();
                        let mode_class = if rule.mode == "sync" { "bg-purple-100 text-purple-800" } else { "bg-blue-100 text-blue-800" };

                        view! {
                            <div class="px-6 py-4">
                                <div class="flex items-center justify-between">
                                    <div class="flex-1">
                                        <div class="flex items-center space-x-3">
                                            <h4 class="text-sm font-medium text-gray-900 dark:text-white">{&rule.source_bucket}</h4>
                                            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"/>
                                            </svg>
                                            <span class="text-sm text-gray-500 dark:text-gray-400">{&rule.destination_bucket}</span>
                                            <span class=format!("inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {}", mode_class)>
                                                {&rule.mode}
                                            </span>
                                            {rule.enabled.then(|| view! {
                                                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                                                    "Active"
                                                </span>
                                            })}
                                        </div>
                                        <div class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                                            {rule.prefix_filter.as_ref().map(|p| format!("Prefix: {} • ", p)).unwrap_or_default()}
                                            {if rule.replicate_deletes { "Deletes: Yes" } else { "Deletes: No" }}
                                            {format!(" • Priority: {}", rule.priority)}
                                        </div>
                                    </div>
                                    <div class="flex items-center space-x-2">
                                        <button
                                            class="text-red-600 hover:text-red-800 p-2"
                                            on:click=move |_| on_delete(rule_id.clone())
                                        >
                                            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                                            </svg>
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>

                // Empty state
                <Show when=move || rules.get().is_empty()>
                    <div class="text-center py-12">
                        <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4"/>
                        </svg>
                        <h3 class="mt-2 text-sm font-medium text-gray-900 dark:text-white">"No replication rules"</h3>
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">"Create a rule to start replicating data."</p>
                    </div>
                </Show>
            </div>
        </div>
    }
}

/// Format bytes to human readable
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

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
