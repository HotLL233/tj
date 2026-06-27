// Library crate for integration tests + shared modules
pub mod config;
pub mod db;
pub mod models;
pub mod repo;
pub mod api;
pub mod audit;
pub mod service;
pub mod error;
// tray module is binary-only (platform-specific + windows_subsystem attributes)

