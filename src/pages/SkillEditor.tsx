import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, Save, RotateCcw, CheckCircle } from 'lucide-react';
import {
  getSkillById,
  getSkillContent,
  saveSkillContent,
} from '../tauri-api';

const SkillEditor: React.FC = () => {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [content, setContent] = useState('');
  const [hasChanges, setHasChanges] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  const { data: skill } = useQuery({
    queryKey: ['skill', id],
    queryFn: () => getSkillById(id!),
    enabled: !!id,
  });

  const { data: skillContent } = useQuery({
    queryKey: ['skillContent', id],
    queryFn: () => getSkillContent(id!),
    enabled: !!id,
  });

  React.useEffect(() => {
    if (skillContent) {
      setContent(skillContent);
      setHasChanges(false);
    }
  }, [skillContent]);

  const saveMutation = useMutation({
    mutationFn: (newContent: string) => saveSkillContent(id!, newContent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skillContent', id] });
      setHasChanges(false);
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
    onError: (error: Error) => {
      alert(t('common.saveFailed') + ': ' + error.message);
    },
  });

  const handleContentChange = (newContent: string) => {
    setContent(newContent);
    setHasChanges(newContent !== skillContent);
    setSaveSuccess(false);
  };

  const handleSave = () => {
    saveMutation.mutate(content);
  };

  return (
    <div className="max-w-4xl mx-auto">
      <div className="flex items-center mb-6">
        <button
          onClick={() => navigate('/settings/skills')}
          className="p-2 -ml-2 text-gray-500 hover:bg-gray-100 rounded-lg mr-2"
        >
          <ArrowLeft className="w-5 h-5" />
        </button>
        <div>
          <h1 className="text-2xl font-bold text-gray-900">{skill?.name}</h1>
          {skill?.description && (
            <p className="text-gray-500">{skill.description}</p>
          )}
        </div>
      </div>

      {saveSuccess && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg flex items-center text-green-700">
          <CheckCircle className="w-5 h-5 mr-2" />
          {t('common.saveSuccess')}
        </div>
      )}

      <div className="bg-white border border-gray-200 rounded-lg">
        <div className="border-b border-gray-200 p-4 flex justify-between items-center">
          <div className="text-sm text-gray-500">
            SKILL.md
          </div>
          <div className="flex items-center space-x-3">
            <button
              onClick={() => handleContentChange(skillContent || '')}
              disabled={!hasChanges}
              className="flex items-center px-3 py-1.5 text-gray-600 hover:text-gray-800 disabled:opacity-50 border border-gray-300 rounded-lg hover:bg-gray-50"
            >
              <RotateCcw className="w-4 h-4 mr-1" />
              {t('common.reset')}
            </button>
            <button
              onClick={handleSave}
              disabled={!hasChanges || saveMutation.isPending}
              className="flex items-center px-4 py-1.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
            >
              <Save className="w-4 h-4 mr-1" />
              {saveMutation.isPending ? t('common.saving') : t('common.save')}
            </button>
          </div>
        </div>

        <textarea
          value={content}
          onChange={(e) => handleContentChange(e.target.value)}
          className="w-full h-[600px] p-4 font-mono text-sm resize-none border-none focus:ring-0"
          spellCheck={false}
        />
      </div>

      <div className="mt-6 bg-gray-50 border border-gray-200 rounded-lg p-4">
        <h3 className="text-sm font-medium text-gray-700 mb-2">Skill {t('skill.description')}</h3>
        <ul className="text-sm text-gray-600 space-y-1">
          <li>• identity: Skill {t('skill.identityDesc')}</li>
          <li>• extract-rules: {t('skill.extractRulesDesc')}</li>
          <li>• priority-rules: {t('skill.priorityRulesDesc')}</li>
          <li>• notify-rules: {t('skill.notifyRulesDesc')}</li>
          <li>• custom-prompt: {t('skill.customPromptDesc')}</li>
        </ul>
      </div>
    </div>
  );
};

export default SkillEditor;
