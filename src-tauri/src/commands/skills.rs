use crate::models::*;
use crate::db::*;
use crate::skills::*;
use tauri::State;
use std::sync::Arc;
use log::info;

#[tauri::command]
pub async fn get_all_skills(state: State<'_, Arc<SkillRepo>>) -> Result<Vec<Skill>, String> {
    info!("[API] get_all_skills called");
    match state.get_all() {
        Ok(skills) => {
            info!("[API] get_all_skills: found {} skills", skills.len());
            Ok(skills)
        }
        Err(e) => {
            info!("[API] get_all_skills error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_skill_by_id(id: String, state: State<'_, Arc<SkillRepo>>) -> Result<Option<Skill>, String> {
    info!("[API] get_skill_by_id called for id: {}", id);
    match state.get_by_id(&id) {
        Ok(skill) => {
            info!("[API] get_skill_by_id: found = {}", skill.is_some());
            Ok(skill)
        }
        Err(e) => {
            info!("[API] get_skill_by_id error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_active_skill(state: State<'_, Arc<SkillRepo>>) -> Result<Option<Skill>, String> {
    info!("[API] get_active_skill called");
    match state.get_active() {
        Ok(skill) => {
            info!("[API] get_active_skill: found = {}", skill.is_some());
            Ok(skill)
        }
        Err(e) => {
            info!("[API] get_active_skill error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn set_active_skill(id: String, state: State<'_, Arc<SkillRepo>>) -> Result<(), String> {
    info!("[API] set_active_skill called for id: {}", id);
    match state.set_active(&id) {
        Ok(_) => {
            info!("[API] set_active_skill: success");
            Ok(())
        }
        Err(e) => {
            info!("[API] set_active_skill error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn create_skill(
    name: String,
    description: String,
    content: String,
    skill_repo: State<'_, Arc<SkillRepo>>,
    skill_loader: State<'_, Arc<SkillLoader>>,
) -> Result<Skill, String> {
    info!("[API] create_skill called for name: {}", name);
    
    info!("[API] Getting existing skills for sort order...");
    let skills = skill_repo.get_all().map_err(|e| e.to_string())?;
    let sort_order = skills.len() as i32;
    info!("[API] New skill sort_order: {}", sort_order);
    
    info!("[API] Creating skill file...");
    let file_path = skill_loader.create_skill(&name, &content).map_err(|e| e.to_string())?;
    info!("[API] Skill file created at: {}", file_path);

    let new_skill = NewSkill {
        name: name.clone(),
        description: description.clone(),
        sort_order,
        is_builtin: false,
        file_path,
    };

    info!("[API] Inserting skill to database...");
    match skill_repo.insert(new_skill) {
        Ok(skill) => {
            info!("[API] create_skill: success, id = {}", skill.id);
            Ok(skill)
        }
        Err(e) => {
            info!("[API] create_skill error: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_skill_content(
    id: String,
    skill_repo: State<'_, Arc<SkillRepo>>,
    skill_loader: State<'_, Arc<SkillLoader>>,
) -> Result<Option<String>, String> {
    info!("[API] get_skill_content called for id: {}", id);
    
    let skill = skill_repo.get_by_id(&id).map_err(|e| e.to_string())?;
    let content = skill.and_then(|s| {
        info!("[API] Loading content for skill: {}", s.name);
        skill_loader.load_skill_content(&s.name)
    });
    info!("[API] get_skill_content: content found = {}", content.is_some());
    Ok(content)
}

#[tauri::command]
pub async fn save_skill_content(
    id: String,
    content: String,
    skill_repo: State<'_, Arc<SkillRepo>>,
    skill_loader: State<'_, Arc<SkillLoader>>,
) -> Result<(), String> {
    info!("[API] save_skill_content called for id: {}", id);
    
    let skill = skill_repo.get_by_id(&id).map_err(|e| e.to_string())?;
    if let Some(skill) = skill {
        info!("[API] Saving content for skill: {}", skill.name);
        skill_loader.save_skill_content(&skill.name, &content).map_err(|e| e.to_string())?;
        skill_repo.touch(&id).map_err(|e| e.to_string())?;
        info!("[API] save_skill_content: success");
    } else {
        info!("[API] save_skill_content: skill not found");
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_skill(
    id: String,
    skill_repo: State<'_, Arc<SkillRepo>>,
    skill_loader: State<'_, Arc<SkillLoader>>,
) -> Result<(), String> {
    info!("[API] delete_skill called for id: {}", id);
    
    let skill = skill_repo.get_by_id(&id).map_err(|e| e.to_string())?;
    if let Some(skill) = skill {
        info!("[API] Deleting skill file for: {} (builtin = {})", skill.name, skill.is_builtin);
        skill_loader.delete_skill(&skill.name, skill.is_builtin).map_err(|e| e.to_string())?;
        info!("[API] Deleting skill from database...");
        skill_repo.delete(&id).map_err(|e| e.to_string())?;
        info!("[API] delete_skill: success");
    } else {
        info!("[API] delete_skill: skill not found");
    }
    Ok(())
}
