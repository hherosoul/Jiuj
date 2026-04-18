export type Priority = 'high' | 'medium' | 'low';
export type ItemType = 'task' | 'deadline' | 'reply' | 'notification' | 'other';
export type ItemStatus = 'pending' | 'overdue';
export type AccountStatus = 'active' | 'disabled';
export type SkipType = 'sender' | 'domain';
export type TrayStatus = 'normal' | 'paused' | 'error';

export type AIProvider =
  | 'openai'
  | 'deepseek'
  | 'kimi'
  | 'zhipu'
  | 'qwen'
  | 'claude'
  | 'ollama'
  | 'custom';

export type Locale = 'zh-CN' | 'en-US' | 'auto';

export type CloseAction = 'minimize-to-tray' | 'quit';

export interface Item {
  id: string;
  content: string;
  deadline?: string;
  time?: string;
  priority: Priority;
  itemType: ItemType;
  remindOffsets?: number[];
  notifiedStages: number[];
  sourceEmailId: string;
  sourceFrom: string;
  sourceSubject: string;
  sourceDate: string;
  sourceAccount: string;
  matchedSkill?: string;
  status: ItemStatus;
  lastNotifiedAt?: string;
  createdAt: string;
  completedAt?: string;
}

export interface Account {
  id: string;
  email: string;
  imapHost: string;
  imapPort: number;
  lastUid: number;
  status: AccountStatus;
}

export interface Skill {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  sortOrder: number;
  isBuiltin: boolean;
  filePath: string;
  updatedAt: string;
}

export interface SkipEntry {
  id: string;
  type: SkipType;
  value: string;
}

export interface Setting {
  key: string;
  value: string;
}

export interface AIConfig {
  provider: AIProvider;
  apiKey: string;
  model: string;
  baseUrl?: string;
  customName?: string;
}

export interface ModelInfo {
  id: string;
  name: string;
}

export interface AIProfile {
  id: string;
  name: string;
  provider: AIProvider;
  model: string;
  baseUrl?: string;
  customName?: string;
  isActive: boolean;
  createdAt: string;
}

export interface AddAIProfileRequest {
  name: string;
  provider: AIProvider;
  model: string;
  baseUrl?: string;
  customName?: string;
  apiKey: string;
}

export interface UpdateAIProfileRequest {
  name?: string;
  model?: string;
  baseUrl?: string | null;
  customName?: string | null;
  apiKey?: string;
}

export interface AIProviderInfo {
  name: string;
  models: ModelInfo[];
  recommendedModel: string;
  baseUrl?: string;
}

export interface SkillTemplate {
  name: string;
  description: string;
  sections: SkillTemplateSections;
}

export interface SkillTemplateSections {
  identity: string;
  extractRules: string;
  priorityRules: string;
  notifyRules: string;
  customPrompt: string;
}

export interface AddAccountRequest {
  email: string;
  imapHost: string;
  imapPort: number;
  password: string;
}

export interface UpdateAccountRequest {
  email?: string;
  imapHost?: string;
  imapPort?: number;
  password?: string;
}

export interface NewSkill {
  name: string;
  description: string;
  sortOrder: number;
  isBuiltin: boolean;
  filePath: string;
}

export interface NewSkipEntry {
  type: SkipType;
  value: string;
}

export interface AccountUpdate {
  email?: string;
  imapHost?: string;
  imapPort?: number;
}
