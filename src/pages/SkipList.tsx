import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Plus, Trash2, User, Globe, CheckCircle } from 'lucide-react';
import ConfirmDialog from '../components/ConfirmDialog';
import {
  getSkipList,
  addSkipEntry,
  deleteSkipEntry,
  NewSkipEntry,
  SkipEntry,
} from '../tauri-api';

const SkipList: React.FC = () => {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [entryType, setEntryType] = useState<'sender' | 'domain'>('sender');
  const [value, setValue] = useState('');
  const [addSuccess, setAddSuccess] = useState(false);
  const [deleteConfirm, setDeleteConfirm] = useState<{ isOpen: boolean; entry: SkipEntry | null }>({
    isOpen: false,
    entry: null,
  });

  const { data: skipList = [] } = useQuery({
    queryKey: ['skipList'],
    queryFn: getSkipList,
  });

  const addMutation = useMutation({
    mutationFn: (entry: NewSkipEntry) => addSkipEntry(entry),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skipList'] });
      setShowForm(false);
      setValue('');
      setAddSuccess(true);
      setTimeout(() => setAddSuccess(false), 3000);
    },
    onError: (error: Error) => {
      alert(t('common.addFailed') + ': ' + error.message);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteSkipEntry,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['skipList'] });
      setDeleteConfirm({ isOpen: false, entry: null });
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (value) {
      addMutation.mutate({ type: entryType, value });
    }
  };

  const handleDeleteClick = (entry: SkipEntry) => {
    setDeleteConfirm({ isOpen: true, entry });
  };

  const handleConfirmDelete = () => {
    if (deleteConfirm.entry) {
      deleteMutation.mutate(deleteConfirm.entry.id);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">{t('settings.skipList')}</h1>
        <button
          onClick={() => setShowForm(true)}
          className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          <Plus className="w-4 h-4 mr-2" />
          {t('common.add')}
        </button>
      </div>

      {addSuccess && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg flex items-center text-green-700">
          <CheckCircle className="w-5 h-5 mr-2" />
          {t('common.addSuccess')}
        </div>
      )}

      {showForm && (
        <div className="bg-white border border-gray-200 rounded-lg p-6 mb-6">
          <form onSubmit={handleSubmit}>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">{t('skipList.type')}</label>
                <div className="flex space-x-4">
                  <label className="flex items-center">
                    <input
                      type="radio"
                      checked={entryType === 'sender'}
                      onChange={() => setEntryType('sender')}
                      className="mr-2"
                    />
                    <User className="w-4 h-4 mr-1 text-gray-500" />
                    {t('skipList.sender')}
                  </label>
                  <label className="flex items-center">
                    <input
                      type="radio"
                      checked={entryType === 'domain'}
                      onChange={() => setEntryType('domain')}
                      className="mr-2"
                    />
                    <Globe className="w-4 h-4 mr-1 text-gray-500" />
                    {t('skipList.domain')}
                  </label>
                </div>
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  {entryType === 'sender' ? t('skipList.emailAddress') : t('skipList.domainName')}
                </label>
                <input
                  type="text"
                  required
                  value={value}
                  onChange={(e) => setValue(e.target.value)}
                  placeholder={entryType === 'sender' ? 'example@email.com' : 'example.com'}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            </div>
            <div className="flex justify-end space-x-3 mt-6">
              <button
                type="button"
                onClick={() => setShowForm(false)}
                className="px-4 py-2 text-gray-600 hover:text-gray-800 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                {t('common.cancel')}
              </button>
              <button
                type="submit"
                disabled={addMutation.isPending}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
              >
                {addMutation.isPending ? t('common.adding') : t('common.add')}
              </button>
            </div>
          </form>
        </div>
      )}

      <div className="space-y-3">
        {skipList.map((entry) => (
          <div key={entry.id} className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center">
                {entry.type === 'sender' ? (
                  <User className="w-5 h-5 text-gray-400 mr-3" />
                ) : (
                  <Globe className="w-5 h-5 text-gray-400 mr-3" />
                )}
                <div>
                  <p className="font-medium text-gray-900">{entry.value}</p>
                  <p className="text-sm text-gray-500">
                    {entry.type === 'sender' ? t('skipList.sender') : t('skipList.domain')}
                  </p>
                </div>
              </div>
              <button
                onClick={() => handleDeleteClick(entry)}
                className="p-2 text-gray-400 hover:text-red-600"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          </div>
        ))}
      </div>

      <ConfirmDialog
        isOpen={deleteConfirm.isOpen}
        title={t('confirm.title')}
        message={deleteConfirm.entry ? t('confirm.deleteSkipEntry', { value: deleteConfirm.entry.value }) : ''}
        confirmText={t('common.delete')}
        cancelText={t('common.cancel')}
        onConfirm={handleConfirmDelete}
        onCancel={() => setDeleteConfirm({ isOpen: false, entry: null })}
        isDanger
      />
    </div>
  );
};

export default SkipList;
