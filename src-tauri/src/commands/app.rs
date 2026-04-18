use crate::services::*;
use tauri::{State, AppHandle};
use std::sync::Arc;
use log::info;

#[tauri::command]
pub async fn trigger_fetch_now(
    scheduler: State<'_, Arc<AppScheduler>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    info!("[API] trigger_fetch_now called");
    scheduler.trigger_fetch_now(app_handle).await;
    info!("[API] trigger_fetch_now: triggered");
    Ok(())
}
