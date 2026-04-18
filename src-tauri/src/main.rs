#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod constants;
mod db;
mod models;
mod services;
mod skills;

use commands::*;
use constants::*;
use db::*;
use models::*;
use services::*;
use skills::*;

use std::sync::Arc;
use tauri::Manager;

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    eprintln!("=== Jiuj Application Starting ===");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            eprintln!("Setup started");
            
            // 使用 Tauri 的沙盒数据目录
            let app_data_dir = app.path().app_local_data_dir()
                .map_err(|e| {
                    eprintln!("Failed to get app local data dir: {}", e);
                    Box::new(e) as Box<dyn std::error::Error>
                })?;
            
            eprintln!("App data directory: {:?}", app_data_dir);

            // 创建目录
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| {
                    eprintln!("Failed to create app data directory: {}", e);
                    Box::new(e)
                })?;
            eprintln!("App data directory created/verified");

            // 创建 skills 子目录
            let skills_dir = app_data_dir.join("skills");
            std::fs::create_dir_all(&skills_dir)
                .map_err(|e| {
                    eprintln!("Failed to create skills directory: {}", e);
                    Box::new(e)
                })?;
            eprintln!("Skills directory: {:?}", skills_dir);

            let app_data_dir_str = app_data_dir.to_string_lossy().to_string();
            let skills_dir_str = skills_dir.to_string_lossy().to_string();
            
            let db_path = app_data_dir.join(DB_NAME);
            let db_path_str = db_path.to_string_lossy().to_string();
            eprintln!("Database path: {}", db_path_str);
            
            // 使用持久化数据库，没有 fallback
            let db = Database::new(&db_path_str)
                .map_err(|e| {
                    eprintln!("Failed to initialize database: {}", e);
                    e
                })?;
            eprintln!("Database initialized successfully");
            let db = Arc::new(db);

            let item_repo = Arc::new(ItemRepo::new(db.clone()));
            let account_repo = Arc::new(AccountRepo::new(db.clone()));
            let skill_repo = Arc::new(SkillRepo::new(db.clone()));
            let skip_list_repo = Arc::new(SkipListRepo::new(db.clone()));
            let settings_repo = Arc::new(SettingsRepo::new(db.clone()));
            let ai_profiles_repo = Arc::new(AIProfilesRepo::new(db.clone()));

            // Check existing data
            match skill_repo.get_all() {
                Ok(skills) => eprintln!("Found {} skills in database", skills.len()),
                Err(e) => eprintln!("Error reading skills: {}", e),
            }
            match settings_repo.get_all(None::<Vec<String>>) {
                Ok(settings) => eprintln!("Found {} settings in database", settings.len()),
                Err(e) => eprintln!("Error reading settings: {}", e),
            }

            let sanitizer = Sanitizer::new();
            let secret_store = Arc::new(SecretStore::new(&app_data_dir_str));
            let skill_loader = Arc::new(SkillLoader::new(&skills_dir_str));

            let mail_fetcher = MailFetcher::new(
                (*account_repo).clone(),
                (*secret_store).clone(),
                sanitizer.clone(),
            );
            let ai_analyzer = AIAnalyzer::new(
                (*ai_profiles_repo).clone(),
                (*settings_repo).clone(),
                (*secret_store).clone(),
                (*skill_loader).clone(),
            );
            let reminder_engine = ReminderEngine::new((*item_repo).clone(), (*settings_repo).clone());
            let app_scheduler = Arc::new(AppScheduler::new(
                (*item_repo).clone(),
                (*account_repo).clone(),
                (*skill_repo).clone(),
                (*skip_list_repo).clone(),
                (*settings_repo).clone(),
                mail_fetcher,
                ai_analyzer,
                reminder_engine,
                (*skill_loader).clone(),
            ));

            setup_tray(&mut *app);

            // 确保默认技能存在
            skill_loader.ensure_builtin_skill(&skills_dir_str)
                .map_err(|e| {
                    eprintln!("Failed to ensure builtin skill: {}", e);
                    e
                })?;

            // 初始化默认技能
            eprintln!("Checking for active skill...");
            if skill_repo.get_active().ok().flatten().is_none() {
                eprintln!("No active skill found, initializing...");
                if let Ok(Some(builtin)) = skill_repo.get_by_name(BUILTIN_SKILL_NAME) {
                    eprintln!("Found existing builtin skill, activating...");
                    let _ = skill_repo.set_active(&builtin.id);
                } else {
                    eprintln!("Creating new builtin skill...");
                    let new_skill = NewSkill {
                        name: BUILTIN_SKILL_NAME.to_string(),
                        description: "通用邮件助手".to_string(),
                        sort_order: 0,
                        is_builtin: true,
                        file_path: skills_dir.join(format!("{}/SKILL.md", BUILTIN_SKILL_NAME)).to_string_lossy().to_string(),
                    };
                    match skill_repo.insert(new_skill) {
                        Ok(skill) => {
                            eprintln!("Created builtin skill with id: {}", skill.id);
                            let _ = skill_repo.set_active(&skill.id);
                        }
                        Err(e) => {
                            eprintln!("Failed to create builtin skill: {}", e);
                            return Err(e);
                        }
                    }
                }
            } else {
                eprintln!("Active skill already exists");
            }
            
            // 确保有 active AI profile
            eprintln!("Checking for active AI profile...");
            if ai_profiles_repo.get_active().ok().flatten().is_none() {
                eprintln!("No active AI profile found, checking existing profiles...");
                if let Ok(all_profiles) = ai_profiles_repo.get_all() {
                    if let Some(first_profile) = all_profiles.first() {
                        eprintln!("Found {} profiles, activating first one: {}", all_profiles.len(), first_profile.id);
                        let _ = ai_profiles_repo.set_active(&first_profile.id);
                    }
                }
            } else {
                eprintln!("Active AI profile already exists");
            }

            eprintln!("Setup completed successfully");

            let app_handle = app.app_handle().clone();

            let scheduler = app_scheduler.clone();
            tauri::async_runtime::spawn(async move {
                scheduler.start_loop(app_handle).await;
            });

            app.manage(item_repo);
            app.manage(account_repo);
            app.manage(skill_repo);
            app.manage(skip_list_repo);
            app.manage(settings_repo);
            app.manage(ai_profiles_repo);
            app.manage(secret_store);
            app.manage(skill_loader);
            app.manage(app_scheduler);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_all_items,
            get_pending_items,
            complete_item,
            ignore_item,
            get_all_accounts,
            add_account,
            update_account,
            delete_account,
            test_account,
            get_ai_providers,
            get_all_ai_profiles,
            get_active_ai_profile,
            add_ai_profile,
            update_ai_profile,
            delete_ai_profile,
            set_active_ai_profile,
            test_ai_profile,
            get_all_skills,
            get_skill_by_id,
            get_active_skill,
            set_active_skill,
            create_skill,
            get_skill_content,
            save_skill_content,
            delete_skill,
            get_settings,
            set_settings,
            get_skip_list,
            add_skip_entry,
            delete_skip_entry,
            get_sanitize_rules,
            get_skill_template,
            test_email_body_sanitization,
            trigger_fetch_now,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let settings_repo = window.state::<Arc<SettingsRepo>>();
                let close_action = settings_repo.get_or("closeAction", "minimize-to-tray");
                if close_action == "quit" {
                    std::process::exit(0);
                } else {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
