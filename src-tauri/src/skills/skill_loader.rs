use crate::constants::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone)]
pub struct SkillLoader {
    skill_dir: PathBuf,
}

impl SkillLoader {
    pub fn new(skill_dir: &str) -> Self {
        let skill_dir = PathBuf::from(shellexpand::tilde(skill_dir).to_string());
        SkillLoader { skill_dir }
    }

    pub fn load_skill_content(&self, skill_name: &str) -> Option<String> {
        let mut path = self.skill_dir.clone();
        path.push(skill_name);
        path.push("SKILL.md");
        fs::read_to_string(path).ok()
    }

    pub fn save_skill_content(&self, skill_name: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = self.skill_dir.clone();
        path.push(skill_name);
        fs::create_dir_all(&path)?;
        path.push("SKILL.md");
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn delete_skill(&self, skill_name: &str, is_builtin: bool) -> Result<(), Box<dyn std::error::Error>> {
        if is_builtin && skill_name == BUILTIN_SKILL_NAME {
            return Err("Cannot delete built-in skill".into());
        }
        let mut path = self.skill_dir.clone();
        path.push(skill_name);
        fs::remove_dir_all(path)?;
        Ok(())
    }

    pub fn create_skill(&self, skill_name: &str, content: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut path = self.skill_dir.clone();
        path.push(skill_name);
        fs::create_dir_all(&path)?;
        path.push("SKILL.md");
        let mut file = File::create(&path)?;
        file.write_all(content.as_bytes())?;
        Ok(path.to_string_lossy().to_string())
    }

    pub fn ensure_builtin_skill(&self, builtin_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut dest_path = self.skill_dir.clone();
        dest_path.push(BUILTIN_SKILL_NAME);
        dest_path.push("SKILL.md");

        if !dest_path.exists() {
            fs::create_dir_all(dest_path.parent().unwrap())?;
            let mut src_path = PathBuf::from(builtin_dir);
            src_path.push("SKILL.md");
            if src_path.exists() {
                fs::copy(src_path, dest_path)?;
            } else {
                let default_content = Self::default_skill_content();
                let mut file = File::create(dest_path)?;
                file.write_all(default_content.as_bytes())?;
            }
        }

        Ok(())
    }

    fn default_skill_content() -> &'static str {
        r#"# Skill: default

## identity
通用邮件助手，从邮件中提取需要关注的事项，判断是否需要显示在看板上。

## extract-rules
只提取以下类型的信息：
- 截止日期（任何需要提交的时间）
- 会议/活动安排（时间 + 地点 + 议题）
- 需要回复的邮件

必须忽略：
- 纯转发邮件
- 节日问候、祝福
- 会议纪要（除非含明确行动项）
- 普通通知、公告（无明确行动项）
- 广告、促销
- 自动回复、系统通知
- 无截止日期且无活动时间的邮件

## visible-rules
visible 为 true 的条件（满足任一即可）：
- 有截止日期（deadline 不为 null）
- 有活动/会议时间（time 不为 null）
- 需要回复（含"请回复""RSVP""请确认"等）

visible 为 false 的条件：
- 普通通知、广告、无明确行动项
- 纯信息告知类邮件
- 无截止日期且无活动时间且无需回复

## priority-rules
- 含"截止""deadline""due"且距到期 ≤ 48h → high
- 含"请回复""RSVP"且来自非 noreply 地址 → medium
- 含"通知""公告"但无明确行动项 → low

## notify-rules
- high 优先级：立即通知
- medium 优先级：正常通知
- low 优先级：不主动通知，仅在看板展示
- 截止日期前 24h、2h 各提醒一次

## custom-prompt

"#
    }
}
