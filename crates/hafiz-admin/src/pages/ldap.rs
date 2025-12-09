//! LDAP/Active Directory Settings Page

use leptos::*;
use crate::api;
use crate::api::types::*;
use crate::components::{Button, ButtonVariant};

#[component]
pub fn LdapSettingsPage() -> impl IntoView {
    // State
    let (loading, set_loading) = create_signal(false);
    let (error_msg, set_error_msg) = create_signal(Option::<String>::None);
    let (success_msg, set_success_msg) = create_signal(Option::<String>::None);

    // LDAP Status resource
    let ldap_status = create_resource(|| (), |_| async move { api::get_ldap_status().await });
    let ldap_config = create_resource(|| (), |_| async move { api::get_ldap_config().await });

    // Form fields
    let (enabled, set_enabled) = create_signal(false);
    let (server_url, set_server_url) = create_signal(String::new());
    let (server_type, set_server_type) = create_signal("ldap".to_string());
    let (start_tls, set_start_tls) = create_signal(false);
    let (skip_tls_verify, set_skip_tls_verify) = create_signal(false);
    let (bind_dn, set_bind_dn) = create_signal(String::new());
    let (bind_password, set_bind_password) = create_signal(String::new());
    let (user_base_dn, set_user_base_dn) = create_signal(String::new());
    let (user_filter, set_user_filter) = create_signal("(uid={username})".to_string());
    let (group_base_dn, set_group_base_dn) = create_signal(String::new());
    let (group_filter, set_group_filter) = create_signal(String::new());
    let (username_attr, set_username_attr) = create_signal("uid".to_string());
    let (email_attr, set_email_attr) = create_signal("mail".to_string());
    let (display_name_attr, set_display_name_attr) = create_signal("cn".to_string());
    let (timeout, set_timeout) = create_signal(10u64);
    let (cache_ttl, set_cache_ttl) = create_signal(300u64);
    let (default_policies, set_default_policies) = create_signal("readonly".to_string());
    let (group_policies_json, set_group_policies_json) = create_signal("{}".to_string());

    // Test fields
    let (test_username, set_test_username) = create_signal(String::new());
    let (test_password, set_test_password) = create_signal(String::new());
    let (test_result, set_test_result) = create_signal(Option::<String>::None);
    let (test_success, set_test_success) = create_signal(false);

    // Load config into form when available
    create_effect(move |_| {
        if let Some(Ok(config)) = ldap_config.get() {
            set_enabled.set(config.enabled);
            set_server_url.set(config.server_url);
            set_server_type.set(config.server_type);
            set_start_tls.set(config.start_tls);
            set_skip_tls_verify.set(config.skip_tls_verify);
            set_bind_dn.set(config.bind_dn);
            set_user_base_dn.set(config.user_base_dn);
            set_user_filter.set(config.user_filter);
            set_group_base_dn.set(config.group_base_dn.unwrap_or_default());
            set_group_filter.set(config.group_filter.unwrap_or_default());
            set_username_attr.set(config.username_attribute);
            set_email_attr.set(config.email_attribute);
            set_display_name_attr.set(config.display_name_attribute);
            set_timeout.set(config.timeout_seconds);
            set_cache_ttl.set(config.cache_ttl_seconds);
            set_default_policies.set(config.default_policies.join(", "));
            if let Ok(json) = serde_json::to_string_pretty(&config.group_policies) {
                set_group_policies_json.set(json);
            }
        }
    });

    // Apply Active Directory defaults
    let apply_ad_defaults = move |_| {
        set_server_type.set("active_directory".to_string());
        set_user_filter.set("(sAMAccountName={username})".to_string());
        set_group_filter.set("(member={dn})".to_string());
        set_username_attr.set("sAMAccountName".to_string());
        set_display_name_attr.set("displayName".to_string());
    };

    // Apply OpenLDAP defaults
    let apply_openldap_defaults = move |_| {
        set_server_type.set("openldap".to_string());
        set_user_filter.set("(uid={username})".to_string());
        set_group_filter.set("(memberUid={username})".to_string());
        set_username_attr.set("uid".to_string());
        set_display_name_attr.set("cn".to_string());
    };

    // Test connection
    let test_connection = move |_| {
        set_test_result.set(None);
        set_loading.set(true);

        let url = server_url.get();
        let dn = bind_dn.get();
        let pw = bind_password.get();
        let tls = start_tls.get();
        let skip_verify = skip_tls_verify.get();

        spawn_local(async move {
            let request = TestLdapConnectionRequest {
                server_url: url,
                bind_dn: dn,
                bind_password: pw,
                start_tls: tls,
                skip_tls_verify: skip_verify,
            };

            match api::test_ldap_connection(&request).await {
                Ok(response) => {
                    set_test_success.set(response.success);
                    let msg = if response.success {
                        let info = response.server_info.map(|i| {
                            format!(" ({})", i.vendor.unwrap_or_else(|| "Unknown".to_string()))
                        }).unwrap_or_default();
                        format!("✓ Connection successful{}", info)
                    } else {
                        format!("✗ {}", response.message)
                    };
                    set_test_result.set(Some(msg));
                }
                Err(e) => {
                    set_test_success.set(false);
                    set_test_result.set(Some(format!("✗ Error: {}", e.message)));
                }
            }
            set_loading.set(false);
        });
    };

    // Test user search
    let test_search = move |_| {
        set_test_result.set(None);
        set_loading.set(true);

        let username = test_username.get();

        spawn_local(async move {
            let request = TestLdapSearchRequest { username };

            match api::test_ldap_search(&request).await {
                Ok(response) => {
                    set_test_success.set(response.success);
                    let msg = if response.success {
                        if let Some(user) = response.user {
                            format!("✓ User found: {} ({})\nDN: {}\nGroups: {}\nPolicies: {}",
                                user.display_name.unwrap_or_else(|| user.username.clone()),
                                user.email.unwrap_or_else(|| "-".to_string()),
                                user.dn,
                                user.groups.join(", "),
                                user.policies.join(", "))
                        } else {
                            "✓ User found".to_string()
                        }
                    } else {
                        format!("✗ {}", response.message)
                    };
                    set_test_result.set(Some(msg));
                }
                Err(e) => {
                    set_test_success.set(false);
                    set_test_result.set(Some(format!("✗ Error: {}", e.message)));
                }
            }
            set_loading.set(false);
        });
    };

    // Test authentication
    let test_auth = move |_| {
        set_test_result.set(None);
        set_loading.set(true);

        let username = test_username.get();
        let password = test_password.get();

        spawn_local(async move {
            let request = TestLdapAuthRequest { username, password };

            match api::test_ldap_auth(&request).await {
                Ok(response) => {
                    set_test_success.set(response.success);
                    let msg = if response.success {
                        if let Some(user) = response.user {
                            format!("✓ Authentication successful!\nUser: {} ({})\nPolicies: {}",
                                user.display_name.unwrap_or_else(|| user.username.clone()),
                                user.email.unwrap_or_else(|| "-".to_string()),
                                user.policies.join(", "))
                        } else {
                            "✓ Authentication successful".to_string()
                        }
                    } else {
                        format!("✗ {}", response.message)
                    };
                    set_test_result.set(Some(msg));
                }
                Err(e) => {
                    set_test_success.set(false);
                    set_test_result.set(Some(format!("✗ Error: {}", e.message)));
                }
            }
            set_loading.set(false);
        });
    };

    // Save configuration
    let save_config = move |_| {
        set_error_msg.set(None);
        set_success_msg.set(None);
        set_loading.set(true);

        // Parse group policies JSON
        let group_policies: std::collections::HashMap<String, Vec<String>> =
            serde_json::from_str(&group_policies_json.get()).unwrap_or_default();

        // Parse default policies
        let default_policies_vec: Vec<String> = default_policies.get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let config = LdapConfig {
            enabled: enabled.get(),
            server_url: server_url.get(),
            server_type: server_type.get(),
            start_tls: start_tls.get(),
            skip_tls_verify: skip_tls_verify.get(),
            bind_dn: bind_dn.get(),
            bind_password: Some(bind_password.get()).filter(|s| !s.is_empty()),
            user_base_dn: user_base_dn.get(),
            user_filter: user_filter.get(),
            group_base_dn: Some(group_base_dn.get()).filter(|s| !s.is_empty()),
            group_filter: Some(group_filter.get()).filter(|s| !s.is_empty()),
            username_attribute: username_attr.get(),
            email_attribute: email_attr.get(),
            display_name_attribute: display_name_attr.get(),
            group_name_attribute: "cn".to_string(),
            timeout_seconds: timeout.get(),
            cache_ttl_seconds: cache_ttl.get(),
            group_policies,
            default_policies: default_policies_vec,
        };

        spawn_local(async move {
            match api::update_ldap_config(&config).await {
                Ok(_) => {
                    set_success_msg.set(Some("LDAP configuration saved successfully".to_string()));
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to save: {}", e.message)));
                }
            }
            set_loading.set(false);
        });
    };

    // Clear cache
    let clear_cache = move |_| {
        spawn_local(async move {
            match api::clear_ldap_cache().await {
                Ok(_) => {
                    set_success_msg.set(Some("LDAP cache cleared".to_string()));
                }
                Err(e) => {
                    set_error_msg.set(Some(format!("Failed to clear cache: {}", e.message)));
                }
            }
        });
    };

    view! {
        <div class="space-y-6">
            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white">"LDAP / Active Directory"</h1>
                    <p class="text-gray-400 mt-1">"Configure enterprise authentication"</p>
                </div>

                // Status indicator
                <Suspense fallback=move || view! { <StatusBadge status="loading" /> }>
                    {move || ldap_status.get().map(|result| match result {
                        Ok(status) => view! {
                            <StatusBadge
                                status=if !status.enabled { "disabled" }
                                       else if status.connected { "connected" }
                                       else { "error" }
                            />
                        }.into_view(),
                        Err(_) => view! { <StatusBadge status="error" /> }.into_view()
                    })}
                </Suspense>
            </div>

            // Messages
            {move || error_msg.get().map(|msg| view! {
                <div class="bg-red-900/50 border border-red-700 text-red-200 px-4 py-3 rounded-lg">
                    {msg}
                </div>
            })}
            {move || success_msg.get().map(|msg| view! {
                <div class="bg-green-900/50 border border-green-700 text-green-200 px-4 py-3 rounded-lg">
                    {msg}
                </div>
            })}

            // Main content
            <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                // Left column - Configuration
                <div class="lg:col-span-2 space-y-6">
                    // Enable/Disable
                    <SettingsCard title="LDAP Authentication" description="Enable or disable LDAP/AD integration">
                        <div class="flex items-center justify-between">
                            <div>
                                <p class="text-white font-medium">"Enable LDAP"</p>
                                <p class="text-sm text-gray-400">"Use LDAP/Active Directory for user authentication"</p>
                            </div>
                            <ToggleSwitch
                                enabled=enabled
                                on_toggle=move |v| set_enabled.set(v)
                            />
                        </div>
                    </SettingsCard>

                    // Server Configuration
                    <SettingsCard title="Server Configuration" description="LDAP server connection settings">
                        // Quick setup buttons
                        <div class="flex gap-2 mb-4">
                            <button
                                class="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-700 text-white rounded-lg"
                                on:click=apply_ad_defaults
                            >
                                "Active Directory"
                            </button>
                            <button
                                class="px-3 py-1.5 text-sm bg-gray-600 hover:bg-gray-700 text-white rounded-lg"
                                on:click=apply_openldap_defaults
                            >
                                "OpenLDAP"
                            </button>
                        </div>

                        <div class="grid grid-cols-2 gap-4">
                            <div class="col-span-2">
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Server URL"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="ldaps://ldap.example.com:636"
                                    prop:value=move || server_url.get()
                                    on:input=move |ev| set_server_url.set(event_target_value(&ev))
                                />
                                <p class="text-xs text-gray-500 mt-1">"Use ldaps:// for SSL or ldap:// with STARTTLS"</p>
                            </div>

                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Server Type"
                                </label>
                                <select
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    prop:value=move || server_type.get()
                                    on:change=move |ev| set_server_type.set(event_target_value(&ev))
                                >
                                    <option value="ldap">"Generic LDAP"</option>
                                    <option value="active_directory">"Active Directory"</option>
                                    <option value="openldap">"OpenLDAP"</option>
                                    <option value="389ds">"389 Directory Server"</option>
                                </select>
                            </div>

                            <div class="flex items-end gap-4">
                                <label class="flex items-center gap-2 text-gray-300">
                                    <input
                                        type="checkbox"
                                        class="w-4 h-4 rounded bg-gray-700 border-gray-600"
                                        prop:checked=move || start_tls.get()
                                        on:change=move |ev| set_start_tls.set(event_target_checked(&ev))
                                    />
                                    "STARTTLS"
                                </label>
                                <label class="flex items-center gap-2 text-gray-300">
                                    <input
                                        type="checkbox"
                                        class="w-4 h-4 rounded bg-gray-700 border-gray-600"
                                        prop:checked=move || skip_tls_verify.get()
                                        on:change=move |ev| set_skip_tls_verify.set(event_target_checked(&ev))
                                    />
                                    "Skip TLS Verify"
                                </label>
                            </div>
                        </div>
                    </SettingsCard>

                    // Bind Credentials
                    <SettingsCard title="Service Account" description="Credentials for LDAP queries">
                        <div class="grid grid-cols-1 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Bind DN"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="cn=admin,dc=example,dc=com"
                                    prop:value=move || bind_dn.get()
                                    on:input=move |ev| set_bind_dn.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Bind Password"
                                </label>
                                <input
                                    type="password"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="••••••••"
                                    prop:value=move || bind_password.get()
                                    on:input=move |ev| set_bind_password.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    </SettingsCard>

                    // User Search Settings
                    <SettingsCard title="User Search" description="How to find users in LDAP">
                        <div class="grid grid-cols-1 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "User Base DN"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="ou=users,dc=example,dc=com"
                                    prop:value=move || user_base_dn.get()
                                    on:input=move |ev| set_user_base_dn.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "User Filter"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="(uid={username})"
                                    prop:value=move || user_filter.get()
                                    on:input=move |ev| set_user_filter.set(event_target_value(&ev))
                                />
                                <p class="text-xs text-gray-500 mt-1">"Use {username} as placeholder. AD: (sAMAccountName={username})"</p>
                            </div>
                        </div>
                    </SettingsCard>

                    // Group Settings
                    <SettingsCard title="Group Settings" description="Group membership and policy mapping">
                        <div class="grid grid-cols-1 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Group Base DN"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="ou=groups,dc=example,dc=com"
                                    prop:value=move || group_base_dn.get()
                                    on:input=move |ev| set_group_base_dn.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Group Filter"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="(member={dn})"
                                    prop:value=move || group_filter.get()
                                    on:input=move |ev| set_group_filter.set(event_target_value(&ev))
                                />
                                <p class="text-xs text-gray-500 mt-1">"Use {dn} for user DN or {username} for username"</p>
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Group → Policy Mapping (JSON)"
                                </label>
                                <textarea
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500 font-mono text-sm"
                                    rows="4"
                                    placeholder=r#"{"admins": ["admin"], "developers": ["readwrite"]}"#
                                    prop:value=move || group_policies_json.get()
                                    on:input=move |ev| set_group_policies_json.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Default Policies"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="readonly"
                                    prop:value=move || default_policies.get()
                                    on:input=move |ev| set_default_policies.set(event_target_value(&ev))
                                />
                                <p class="text-xs text-gray-500 mt-1">"Comma-separated policies for users without group mapping"</p>
                            </div>
                        </div>
                    </SettingsCard>

                    // Attribute Mapping
                    <SettingsCard title="Attribute Mapping" description="Map LDAP attributes to user fields">
                        <div class="grid grid-cols-3 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Username"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    prop:value=move || username_attr.get()
                                    on:input=move |ev| set_username_attr.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Email"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    prop:value=move || email_attr.get()
                                    on:input=move |ev| set_email_attr.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Display Name"
                                </label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    prop:value=move || display_name_attr.get()
                                    on:input=move |ev| set_display_name_attr.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                    </SettingsCard>

                    // Advanced Settings
                    <SettingsCard title="Advanced" description="Timeout and caching settings">
                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Connection Timeout (seconds)"
                                </label>
                                <input
                                    type="number"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    prop:value=move || timeout.get().to_string()
                                    on:input=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse() {
                                            set_timeout.set(v);
                                        }
                                    }
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">
                                    "Cache TTL (seconds)"
                                </label>
                                <input
                                    type="number"
                                    class="w-full px-4 py-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    prop:value=move || cache_ttl.get().to_string()
                                    on:input=move |ev| {
                                        if let Ok(v) = event_target_value(&ev).parse() {
                                            set_cache_ttl.set(v);
                                        }
                                    }
                                />
                            </div>
                        </div>
                    </SettingsCard>

                    // Save button
                    <div class="flex justify-end gap-4">
                        <Button
                            variant=ButtonVariant::Secondary
                            on_click=clear_cache
                        >
                            "Clear Cache"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            on_click=save_config
                            disabled=loading
                        >
                            {move || if loading.get() { "Saving..." } else { "Save Configuration" }}
                        </Button>
                    </div>
                </div>

                // Right column - Status & Testing
                <div class="space-y-6">
                    // Connection Status
                    <SettingsCard title="Connection Status" description="Current LDAP connection state">
                        <Suspense fallback=move || view! { <div class="animate-pulse h-20 bg-gray-700 rounded"></div> }>
                            {move || ldap_status.get().map(|result| match result {
                                Ok(status) => view! {
                                    <div class="space-y-3">
                                        <StatusRow label="Enabled" value=if status.enabled { "Yes" } else { "No" } ok=status.enabled />
                                        <StatusRow label="Connected" value=if status.connected { "Yes" } else { "No" } ok=status.connected />
                                        <StatusRow label="Server" value=&status.server_url ok=true />
                                        <StatusRow label="Cached Users" value=&status.cached_users.to_string() ok=true />
                                        {status.error.map(|e| view! {
                                            <div class="p-2 bg-red-900/30 rounded text-red-300 text-sm">
                                                {e}
                                            </div>
                                        })}
                                    </div>
                                }.into_view(),
                                Err(e) => view! {
                                    <p class="text-red-400">{format!("Error: {}", e.message)}</p>
                                }.into_view()
                            })}
                        </Suspense>
                    </SettingsCard>

                    // Test Panel
                    <SettingsCard title="Test Connection" description="Verify LDAP settings">
                        <div class="space-y-4">
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=test_connection
                                disabled=loading
                                class="w-full"
                            >
                                "Test Connection"
                            </Button>

                            <div class="border-t border-gray-700 pt-4">
                                <p class="text-sm text-gray-400 mb-2">"Test User Search / Auth"</p>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 mb-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="Username"
                                    prop:value=move || test_username.get()
                                    on:input=move |ev| set_test_username.set(event_target_value(&ev))
                                />
                                <input
                                    type="password"
                                    class="w-full px-4 py-2 mb-2 bg-gray-700 border border-gray-600 rounded-lg
                                           text-white focus:outline-none focus:border-blue-500"
                                    placeholder="Password (for auth test)"
                                    prop:value=move || test_password.get()
                                    on:input=move |ev| set_test_password.set(event_target_value(&ev))
                                />
                                <div class="flex gap-2">
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        on_click=test_search
                                        disabled=loading
                                        class="flex-1"
                                    >
                                        "Search"
                                    </Button>
                                    <Button
                                        variant=ButtonVariant::Primary
                                        on_click=test_auth
                                        disabled=loading
                                        class="flex-1"
                                    >
                                        "Authenticate"
                                    </Button>
                                </div>
                            </div>

                            // Test result
                            {move || test_result.get().map(|result| view! {
                                <div class=move || format!(
                                    "mt-4 p-3 rounded-lg text-sm whitespace-pre-wrap {}",
                                    if test_success.get() { "bg-green-900/30 text-green-300" } else { "bg-red-900/30 text-red-300" }
                                )>
                                    {result}
                                </div>
                            })}
                        </div>
                    </SettingsCard>

                    // Help
                    <SettingsCard title="Quick Reference" description="Common configurations">
                        <div class="space-y-3 text-sm">
                            <div class="p-3 bg-gray-750 rounded-lg">
                                <p class="text-blue-400 font-medium">"Active Directory"</p>
                                <p class="text-gray-400">"Filter: (sAMAccountName={username})"</p>
                                <p class="text-gray-400">"Groups: (member={dn})"</p>
                            </div>
                            <div class="p-3 bg-gray-750 rounded-lg">
                                <p class="text-green-400 font-medium">"OpenLDAP"</p>
                                <p class="text-gray-400">"Filter: (uid={username})"</p>
                                <p class="text-gray-400">"Groups: (memberUid={username})"</p>
                            </div>
                        </div>
                    </SettingsCard>
                </div>
            </div>
        </div>
    }
}

// Helper components

#[component]
fn SettingsCard(
    title: &'static str,
    description: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="bg-gray-800 rounded-xl border border-gray-700 p-6">
            <div class="mb-4">
                <h2 class="text-lg font-semibold text-white">{title}</h2>
                <p class="text-sm text-gray-400">{description}</p>
            </div>
            {children()}
        </div>
    }
}

#[component]
fn StatusBadge(status: &'static str) -> impl IntoView {
    let (bg, text, label) = match status {
        "connected" => ("bg-green-900/50", "text-green-400", "Connected"),
        "disabled" => ("bg-gray-700", "text-gray-400", "Disabled"),
        "loading" => ("bg-blue-900/50", "text-blue-400", "Loading..."),
        _ => ("bg-red-900/50", "text-red-400", "Error"),
    };

    view! {
        <span class=format!("px-3 py-1 rounded-full text-sm font-medium {} {}", bg, text)>
            {label}
        </span>
    }
}

#[component]
fn StatusRow(label: &'static str, value: &str, ok: bool) -> impl IntoView {
    view! {
        <div class="flex justify-between items-center">
            <span class="text-gray-400">{label}</span>
            <span class=if ok { "text-white" } else { "text-red-400" }>{value.to_string()}</span>
        </div>
    }
}

#[component]
fn ToggleSwitch(
    enabled: ReadSignal<bool>,
    on_toggle: impl Fn(bool) + 'static,
) -> impl IntoView {
    view! {
        <button
            class=move || {
                let base = "relative inline-flex h-6 w-11 items-center rounded-full transition-colors";
                if enabled.get() {
                    format!("{} bg-blue-600", base)
                } else {
                    format!("{} bg-gray-600", base)
                }
            }
            on:click=move |_| on_toggle(!enabled.get())
        >
            <span
                class=move || {
                    let base = "inline-block h-4 w-4 transform rounded-full bg-white transition-transform";
                    if enabled.get() {
                        format!("{} translate-x-6", base)
                    } else {
                        format!("{} translate-x-1", base)
                    }
                }
            />
        </button>
    }
}

fn event_target_value(ev: &web_sys::Event) -> String {
    use wasm_bindgen::JsCast;
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|e| e.value())
        .or_else(|| {
            ev.target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                .map(|e| e.value())
        })
        .or_else(|| {
            ev.target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                .map(|e| e.value())
        })
        .unwrap_or_default()
}

fn event_target_checked(ev: &web_sys::Event) -> bool {
    use wasm_bindgen::JsCast;
    ev.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|e| e.checked())
        .unwrap_or(false)
}
