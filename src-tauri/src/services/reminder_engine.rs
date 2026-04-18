use crate::constants::*;
use crate::models::*;
use crate::db::ItemRepo;
use crate::db::SettingsRepo;
use chrono::{Utc, Duration, TimeZone, Datelike, Timelike};

pub struct ReminderEngine {
    item_repo: ItemRepo,
    settings_repo: SettingsRepo,
}

impl ReminderEngine {
    pub fn new(item_repo: ItemRepo, settings_repo: SettingsRepo) -> Self {
        ReminderEngine { item_repo, settings_repo }
    }

    fn get_remind_offsets(&self) -> Vec<u64> {
        self.settings_repo
            .get_or("defaultRemindOffsets", &DEFAULT_REMIND_OFFSETS.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","))
            .split(',')
            .filter_map(|s| s.trim().parse::<u64>().ok())
            .collect()
    }

    fn is_date_only(dt: &chrono::DateTime<chrono::FixedOffset>) -> bool {
        dt.hour() == 0 && dt.minute() == 0 && dt.second() == 0
    }

    fn get_date_only_remind_time(deadline: &chrono::DateTime<chrono::FixedOffset>) -> chrono::DateTime<Utc> {
        let deadline_utc = deadline.with_timezone(&Utc);
        let remind_date = deadline_utc.date_naive() - chrono::Duration::days(1);
        Utc.with_ymd_and_hms(remind_date.year(), remind_date.month(), remind_date.day(), 10, 0, 0)
            .single()
            .unwrap_or_else(|| deadline_utc - Duration::days(1))
    }

    pub fn check_deadlines(&self) -> (Vec<Item>, usize) {
        if let Err(e) = self.item_repo.mark_overdue() {
            log::error!("Failed to mark overdue items: {}", e);
        }

        let pending_items = match self.item_repo.get_pending() {
            Ok(items) => items,
            Err(e) => {
                log::error!("Failed to get pending items: {}", e);
                return (vec![], 0);
            }
        };

        let default_offsets = self.get_remind_offsets();

        let mut items_to_notify = Vec::new();
        let mut overdue_count = 0;

        for item in pending_items {
            if item.status == ItemStatus::Overdue {
                overdue_count += 1;
            }

            if let Some(deadline_str) = &item.deadline {
                if let Ok(deadline) = chrono::DateTime::parse_from_rfc3339(deadline_str) {
                    let now = Utc::now();

                    if Self::is_date_only(&deadline) {
                        let remind_time = Self::get_date_only_remind_time(&deadline);
                        if !item.notified_stages.contains(&0) && remind_time <= now {
                            items_to_notify.push(item.clone());
                            if let Err(e) = self.mark_notified(&item.id, 0) {
                                log::error!("Failed to mark stage 0 notified for item {}: {}", item.id, e);
                            }
                        }
                    } else {
                        let deadline_utc = deadline.with_timezone(&Utc);
                        let offsets = item.remind_offsets.as_deref()
                            .filter(|o| !o.is_empty())
                            .unwrap_or(&default_offsets);

                        let mut triggered_stage: Option<usize> = None;

                        for (stage_idx, &offset_minutes) in offsets.iter().enumerate() {
                            if item.notified_stages.contains(&stage_idx) {
                                continue;
                            }

                            let remind_time = deadline_utc - Duration::minutes(offset_minutes as i64);

                            if remind_time <= now {
                                triggered_stage = Some(stage_idx);
                            } else {
                                break;
                            }
                        }

                        if let Some(stage) = triggered_stage {
                            items_to_notify.push(item.clone());
                            if let Err(e) = self.mark_notified(&item.id, stage) {
                                log::error!("Failed to mark stage {} notified for item {}: {}", stage, item.id, e);
                            }
                        }
                    }
                } else {
                    log::warn!("Failed to parse deadline '{}' for item '{}'", deadline_str, item.content);
                }
            }
        }

        (items_to_notify, overdue_count)
    }

    pub fn mark_notified(&self, item_id: &str, stage: usize) -> Result<(), Box<dyn std::error::Error>> {
        let item = self.item_repo.get_by_id(item_id)?;
        if let Some(item) = item {
            let mut notified_stages = item.notified_stages.clone();
            for s in 0..=stage {
                if !notified_stages.contains(&s) {
                    notified_stages.push(s);
                }
            }
            self.item_repo.update_notified_stages(item_id, &notified_stages)?;
        }
        Ok(())
    }
}
