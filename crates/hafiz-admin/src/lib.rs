//! Hafiz Admin UI
//!
//! A Leptos-based web interface for managing Hafiz.

pub mod api;
pub mod app;
pub mod components;
pub mod pages;

use wasm_bindgen::prelude::*;

/// Initialize and mount the Leptos application
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging
    let _ = console_log::init_with_level(log::Level::Debug);

    log::info!("Hafiz Admin UI starting...");

    // Mount the app
    leptos::mount_to_body(app::App);
}
