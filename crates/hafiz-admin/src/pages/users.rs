//! Users management page

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::api::{self, UserInfo};
use crate::components::{Button, ButtonVariant, Modal};

#[component]
pub fn UsersPage() -> impl IntoView {
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (refresh_trigger, set_refresh_trigger) = create_signal(0);
    
    let users = create_resource(
        move || refresh_trigger.get(),
        |_| async move { api::list_users().await }
    );

    let on_refresh = move || {
        set_refresh_trigger.update(|t| *t += 1);
    };

    view! {
        <div class="space-y-6">
            // Page header
            <div class="flex items-center justify-between">
                <div>
                    <h1 class="text-2xl font-bold text-white">"Users"</h1>
                    <p class="text-gray-400 mt-1">"Manage access credentials and permissions"</p>
                </div>
                <Button on_click=Callback::new(move |_| set_show_create_modal.set(true))>
                    <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                            d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z" />
                    </svg>
                    "Create User"
                </Button>
            </div>

            // Users table
            <div class="bg-gray-800 rounded-xl border border-gray-700 overflow-hidden">
                <Suspense fallback=move || view! { <UsersTableSkeleton /> }>
                    {move || users.get().map(|result| match result {
                        Ok(list) => {
                            if list.is_empty() {
                                view! {
                                    <div class="p-12 text-center">
                                        <svg class="w-16 h-16 mx-auto text-gray-600 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
                                                d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                                        </svg>
                                        <h3 class="text-xl font-semibold text-white mb-2">"No users yet"</h3>
                                        <p class="text-gray-400 mb-6">"Create users to grant access to your storage"</p>
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <table class="w-full">
                                        <thead>
                                            <tr class="border-b border-gray-700 bg-gray-750">
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"User"</th>
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"Access Key"</th>
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"Status"</th>
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"Created"</th>
                                                <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody class="divide-y divide-gray-700">
                                            {list.into_iter().map(|user| {
                                                view! {
                                                    <UserRow user=user on_refresh=move |_| on_refresh() />
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_view()
                            }
                        }
                        Err(e) => view! {
                            <div class="p-6 text-center text-red-400">
                                "Failed to load users: " {e.to_string()}
                            </div>
                        }.into_view()
                    })}
                </Suspense>
            </div>

            // Create user modal
            <CreateUserModal 
                show=show_create_modal
                on_close=Callback::new(move |_| set_show_create_modal.set(false))
                on_created=Callback::new(move |_| {
                    on_refresh();
                    set_show_create_modal.set(false);
                })
            />
        </div>
    }
}

#[component]
fn UserRow(
    user: UserInfo,
    #[prop(into)] on_refresh: Callback<()>,
) -> impl IntoView {
    use wasm_bindgen_futures::spawn_local;
    
    let access_key = user.access_key.clone();
    let access_key_for_delete = access_key.clone();
    let access_key_for_toggle = access_key.clone();
    let is_enabled = user.enabled;
    let user_name = user.name.clone();
    
    let (is_deleting, set_is_deleting) = create_signal(false);
    let (is_toggling, set_is_toggling) = create_signal(false);
    
    let handle_delete = move |_| {
        let key = access_key_for_delete.clone();
        let name = user_name.clone();
        let on_refresh = on_refresh.clone();
        
        let window = web_sys::window().unwrap();
        if !window.confirm_with_message(&format!("Delete user '{}'?\n\nThis will revoke all access for this user.", name)).unwrap_or(false) {
            return;
        }
        
        set_is_deleting.set(true);
        spawn_local(async move {
            match api::delete_credentials(&key).await {
                Ok(_) => on_refresh.call(()),
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
    
    let handle_toggle = move |_| {
        let key = access_key_for_toggle.clone();
        let new_status = !is_enabled;
        let on_refresh = on_refresh.clone();
        
        set_is_toggling.set(true);
        spawn_local(async move {
            match api::update_credentials(&key, new_status).await {
                Ok(_) => on_refresh.call(()),
                Err(e) => {
                    web_sys::window()
                        .unwrap()
                        .alert_with_message(&format!("Update failed: {}", e.message))
                        .ok();
                }
            }
            set_is_toggling.set(false);
        });
    };
    
    view! {
        <tr class="hover:bg-gray-750 transition-colors">
            <td class="px-4 py-3">
                <div class="flex items-center space-x-3">
                    <div class="w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center">
                        <span class="text-white font-medium">
                            {user.name.chars().next().unwrap_or('U').to_uppercase().to_string()}
                        </span>
                    </div>
                    <div>
                        <p class="text-white font-medium">{&user.name}</p>
                        <p class="text-sm text-gray-400">{&user.email.clone().unwrap_or_default()}</p>
                    </div>
                </div>
            </td>
            <td class="px-4 py-3">
                <code class="text-sm text-gray-300 bg-gray-700 px-2 py-1 rounded">
                    {&user.access_key}
                </code>
            </td>
            <td class="px-4 py-3">
                <button
                    class="cursor-pointer"
                    disabled=is_toggling
                    on:click=handle_toggle
                    title=move || if user.enabled { "Click to disable" } else { "Click to enable" }
                >
                    {if user.enabled {
                        view! {
                            <span class="px-2 py-1 text-xs bg-green-600/20 text-green-400 rounded hover:bg-green-600/30 transition-colors">
                                {move || if is_toggling.get() { "..." } else { "Active" }}
                            </span>
                        }
                    } else {
                        view! {
                            <span class="px-2 py-1 text-xs bg-red-600/20 text-red-400 rounded hover:bg-red-600/30 transition-colors">
                                {move || if is_toggling.get() { "..." } else { "Disabled" }}
                            </span>
                        }
                    }}
                </button>
            </td>
            <td class="px-4 py-3 text-gray-400 text-sm">
                {format_date(&user.created_at)}
            </td>
            <td class="px-4 py-3">
                <div class="flex items-center space-x-2">
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
fn CreateUserModal(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (creating, set_creating) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (created_user, set_created_user) = create_signal(Option::<(String, String)>::None);

    let on_submit = move |_| {
        set_creating.set(true);
        set_error.set(None);
        
        let n = name.get();
        let e = email.get();
        
        spawn_local(async move {
            match api::create_user(&n, if e.is_empty() { None } else { Some(&e) }).await {
                Ok((access_key, secret_key)) => {
                    set_created_user.set(Some((access_key, secret_key)));
                }
                Err(e) => {
                    set_error.set(Some(e.to_string()));
                }
            }
            set_creating.set(false);
        });
    };

    view! {
        <Modal title="Create User" show=show on_close=on_close>
            {move || {
                if let Some((access_key, secret_key)) = created_user.get() {
                    view! {
                        <div class="space-y-4">
                            <div class="bg-green-900/20 border border-green-500 rounded-lg p-4">
                                <p class="text-green-400 font-medium">"User created successfully!"</p>
                            </div>
                            
                            <div class="bg-gray-700 rounded-lg p-4 space-y-3">
                                <div>
                                    <label class="text-sm text-gray-400">"Access Key"</label>
                                    <code class="block mt-1 text-white bg-gray-800 px-3 py-2 rounded">
                                        {&access_key}
                                    </code>
                                </div>
                                <div>
                                    <label class="text-sm text-gray-400">"Secret Key"</label>
                                    <code class="block mt-1 text-white bg-gray-800 px-3 py-2 rounded">
                                        {&secret_key}
                                    </code>
                                </div>
                            </div>
                            
                            <p class="text-sm text-yellow-400">
                                "⚠️ Save these credentials now. The secret key cannot be retrieved later."
                            </p>
                            
                            <div class="flex justify-end pt-4">
                                <Button on_click=Callback::new(move |_| {
                                    set_created_user.set(None);
                                    set_name.set(String::new());
                                    set_email.set(String::new());
                                    on_created.call(());
                                })>
                                    "Done"
                                </Button>
                            </div>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="space-y-4">
                            {move || error.get().map(|e| view! {
                                <div class="bg-red-900/50 border border-red-500 text-red-200 px-4 py-3 rounded">
                                    {e}
                                </div>
                            })}
                            
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">"Username"</label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg 
                                           text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                                    placeholder="john"
                                    prop:value=move || name.get()
                                    on:input=move |ev| set_name.set(event_target_value(&ev))
                                />
                            </div>
                            
                            <div>
                                <label class="block text-sm font-medium text-gray-300 mb-2">"Email (optional)"</label>
                                <input
                                    type="email"
                                    class="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-lg 
                                           text-white placeholder-gray-400 focus:outline-none focus:border-blue-500"
                                    placeholder="john@example.com"
                                    prop:value=move || email.get()
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                />
                            </div>

                            <div class="flex justify-end space-x-3 pt-4">
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| on_close.call(()))>
                                    "Cancel"
                                </Button>
                                <Button loading=Some(creating.into()) on_click=Callback::new(on_submit)>
                                    "Create"
                                </Button>
                            </div>
                        </div>
                    }.into_view()
                }
            }}
        </Modal>
    }
}

#[component]
fn UsersTableSkeleton() -> impl IntoView {
    view! {
        <div class="animate-pulse">
            <div class="border-b border-gray-700 px-4 py-3 bg-gray-750">
                <div class="h-4 w-full bg-gray-700 rounded"></div>
            </div>
            {(0..5).map(|_| view! {
                <div class="border-b border-gray-700 px-4 py-3 flex items-center space-x-4">
                    <div class="w-10 h-10 bg-gray-700 rounded-full"></div>
                    <div class="flex-1 space-y-2">
                        <div class="h-4 w-32 bg-gray-700 rounded"></div>
                        <div class="h-3 w-48 bg-gray-700 rounded"></div>
                    </div>
                </div>
            }).collect_view()}
        </div>
    }
}

fn format_date(date: &str) -> String {
    if date.len() >= 10 {
        date[..10].to_string()
    } else {
        date.to_string()
    }
}
