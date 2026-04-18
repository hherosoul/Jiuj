import { invoke } from '@tauri-apps/api/core';
import type * as T from './types';

export * from './types';

export async function getAllItems(): Promise<T.Item[]> {
  return invoke('get_all_items');
}

export async function getPendingItems(): Promise<T.Item[]> {
  return invoke('get_pending_items');
}

export async function completeItem(id: string): Promise<void> {
  return invoke('complete_item', { id });
}

export async function ignoreItem(id: string): Promise<void> {
  return invoke('ignore_item', { id });
}

export async function getAllAccounts(): Promise<T.Account[]> {
  return invoke('get_all_accounts');
}

export async function addAccount(data: T.AddAccountRequest): Promise<T.Account> {
  return invoke('add_account', { data });
}

export async function updateAccount(
  id: string,
  data: T.UpdateAccountRequest
): Promise<void> {
  return invoke('update_account', { id, data });
}

export async function deleteAccount(id: string): Promise<void> {
  return invoke('delete_account', { id });
}

export async function testAccount(
  email: string,
  imapHost: string,
  imapPort: number,
  password: string
): Promise<boolean> {
  return invoke('test_account', { email, imapHost, imapPort, password });
}

export async function getAIProviders(): Promise<[string, T.AIProviderInfo][]> {
  console.log('[tauri-api] getAIProviders called');
  const result = await invoke('get_ai_providers') as [string, T.AIProviderInfo][];
  console.log('[tauri-api] getAIProviders result:', result);
  return result;
}

export async function getAllAIProfiles(): Promise<T.AIProfile[]> {
  console.log('[tauri-api] getAllAIProfiles called');
  const result = await invoke('get_all_ai_profiles') as T.AIProfile[];
  console.log('[tauri-api] getAllAIProfiles result:', result);
  return result;
}

export async function getActiveAIProfile(): Promise<T.AIProfile | null> {
  console.log('[tauri-api] getActiveAIProfile called');
  const result = await invoke('get_active_ai_profile') as T.AIProfile | null;
  console.log('[tauri-api] getActiveAIProfile result:', result);
  return result;
}

export async function addAIProfile(data: T.AddAIProfileRequest): Promise<T.AIProfile> {
  console.log('[tauri-api] addAIProfile called with:', data);
  const result = await invoke('add_ai_profile', { data }) as T.AIProfile;
  console.log('[tauri-api] addAIProfile result:', result);
  return result;
}

export async function updateAIProfile(id: string, data: T.UpdateAIProfileRequest): Promise<void> {
  console.log('[tauri-api] updateAIProfile called for id:', id, 'with data:', data);
  await invoke('update_ai_profile', { id, data });
  console.log('[tauri-api] updateAIProfile completed');
}

export async function deleteAIProfile(id: string): Promise<void> {
  console.log('[tauri-api] deleteAIProfile called for id:', id);
  await invoke('delete_ai_profile', { id });
  console.log('[tauri-api] deleteAIProfile completed');
}

export async function setActiveAIProfile(id: string): Promise<void> {
  console.log('[tauri-api] setActiveAIProfile called for id:', id);
  await invoke('set_active_ai_profile', { id });
  console.log('[tauri-api] setActiveAIProfile completed');
}

export async function testAIProfile(data: T.AddAIProfileRequest): Promise<boolean> {
  console.log('[tauri-api] testAIProfile called with:', data);
  const result = await invoke('test_ai_profile', { data }) as boolean;
  console.log('[tauri-api] testAIProfile result:', result);
  return result;
}

export async function getAllSkills(): Promise<T.Skill[]> {
  return invoke('get_all_skills');
}

export async function getSkillById(id: string): Promise<T.Skill | null> {
  return invoke('get_skill_by_id', { id });
}

export async function getActiveSkill(): Promise<T.Skill | null> {
  return invoke('get_active_skill');
}

export async function setActiveSkill(id: string): Promise<void> {
  return invoke('set_active_skill', { id });
}

export async function createSkill(
  name: string,
  description: string,
  content: string
): Promise<T.Skill> {
  return invoke('create_skill', { name, description, content });
}

export async function getSkillContent(id: string): Promise<string | null> {
  return invoke('get_skill_content', { id });
}

export async function saveSkillContent(id: string, content: string): Promise<void> {
  return invoke('save_skill_content', { id, content });
}

export async function deleteSkill(id: string): Promise<void> {
  return invoke('delete_skill', { id });
}

export async function getSettings(): Promise<T.Setting[]> {
  return invoke('get_settings');
}

export async function setSetting(key: string, value: string): Promise<void> {
  return invoke('set_settings', { key, value });
}

export async function getSkipList(): Promise<T.SkipEntry[]> {
  return invoke('get_skip_list');
}

export async function addSkipEntry(entry: T.NewSkipEntry): Promise<T.SkipEntry> {
  return invoke('add_skip_entry', { entry });
}

export async function deleteSkipEntry(id: string): Promise<void> {
  return invoke('delete_skip_entry', { id });
}

export async function getSanitizeRules(): Promise<[string, string, string][]> {
  return invoke('get_sanitize_rules');
}

export async function getSkillTemplate(): Promise<T.SkillTemplate> {
  return invoke('get_skill_template');
}

export async function testEmailBodySanitization(body: string): Promise<[string, boolean]> {
  return invoke('test_email_body_sanitization', { body });
}

export async function triggerFetchNow(): Promise<void> {
  return invoke('trigger_fetch_now');
}
