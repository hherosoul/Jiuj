use crate::models::*;
use crate::db::*;
use tauri::State;
use std::sync::Arc;
use log::info;

#[tauri::command]
pub async fn get_all_items(state: State<'_, Arc<ItemRepo>>) -> Result<Vec<Item>, String> {
    info!("[API] get_all_items called");
    match state.get_all() {
        Ok(items) => {
            info!("[API] get_all_items: found {} items", items.len());
            Ok(items)
        }
        Err(e) => {
            info!("[API] get_all_items error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_pending_items(state: State<'_, Arc<ItemRepo>>) -> Result<Vec<Item>, String> {
    info!("[API] get_pending_items called");
    match state.get_pending() {
        Ok(items) => {
            info!("[API] get_pending_items: found {} items", items.len());
            Ok(items)
        }
        Err(e) => {
            info!("[API] get_pending_items error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn complete_item(id: String, state: State<'_, Arc<ItemRepo>>) -> Result<(), String> {
    info!("[API] complete_item called for id: {}", id);
    match state.complete(&id) {
        Ok(_) => {
            info!("[API] complete_item: success");
            Ok(())
        }
        Err(e) => {
            info!("[API] complete_item error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn ignore_item(id: String, state: State<'_, Arc<ItemRepo>>) -> Result<(), String> {
    info!("[API] ignore_item called for id: {}", id);
    match state.ignore(&id) {
        Ok(_) => {
            info!("[API] ignore_item: success");
            Ok(())
        }
        Err(e) => {
            info!("[API] ignore_item error: {}", e);
            Err(e.to_string())
        }
    }
}
