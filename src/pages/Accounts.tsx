import React, { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { Plus, Edit, Trash2, Mail, CheckCircle, XCircle } from 'lucide-react';
import ConfirmDialog from '../components/ConfirmDialog';
import {
  getAllAccounts,
  addAccount,
  updateAccount,
  deleteAccount,
  testAccount,
  Account,
  AddAccountRequest,
  UpdateAccountRequest,
} from '../tauri-api';

const Accounts: React.FC = () => {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formData, setFormData] = useState<Partial<AddAccountRequest>>({
    email: '',
    imapHost: '',
    imapPort: 993,
    password: '',
  });
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<{ isOpen: boolean; accountId: string | null; accountEmail: string }>({
    isOpen: false,
    accountId: null,
    accountEmail: '',
  });
  const [saveSuccess, setSaveSuccess] = useState(false);

  const { data: accounts = [] } = useQuery({
    queryKey: ['accounts'],
    queryFn: getAllAccounts,
  });

  const addMutation = useMutation({
    mutationFn: addAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] });
      setShowForm(false);
      setFormData({ email: '', imapHost: '', imapPort: 993, password: '' });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
    onError: (error: Error) => {
      alert(t('common.saveFailed') + ': ' + error.message);
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateAccountRequest }) =>
      updateAccount(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] });
      setEditingId(null);
      setFormData({ email: '', imapHost: '', imapPort: 993, password: '' });
      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    },
    onError: (error: Error) => {
      alert(t('common.saveFailed') + ': ' + error.message);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: deleteAccount,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['accounts'] });
      setDeleteConfirm({ isOpen: false, accountId: null, accountEmail: '' });
    },
  });

  const handleTest = async () => {
    if (!formData.email || !formData.imapHost || !formData.password) return;
    setTesting(true);
    setTestResult(null);
    try {
      await testAccount(formData.email, formData.imapHost, formData.imapPort || 993, formData.password);
      setTestResult({ success: true, message: t('account.testSuccess') });
    } catch (e) {
      setTestResult({ success: false, message: t('account.testFailed') + ': ' + e });
    } finally {
      setTesting(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (editingId) {
      updateMutation.mutate({ id: editingId, data: formData as UpdateAccountRequest });
    } else {
      addMutation.mutate(formData as AddAccountRequest);
    }
  };

  const handleEdit = (account: Account) => {
    setEditingId(account.id);
    setFormData({
      email: account.email,
      imapHost: account.imapHost,
      imapPort: account.imapPort,
    });
    setTestResult(null);
  };

  const handleCancel = () => {
    setShowForm(false);
    setEditingId(null);
    setFormData({ email: '', imapHost: '', imapPort: 993, password: '' });
    setTestResult(null);
  };

  const handleDeleteClick = (account: Account) => {
    setDeleteConfirm({
      isOpen: true,
      accountId: account.id,
      accountEmail: account.email,
    });
  };

  const handleConfirmDelete = () => {
    if (deleteConfirm.accountId) {
      deleteMutation.mutate(deleteConfirm.accountId);
    }
  };

  return (
    <div className="max-w-2xl mx-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold text-gray-900">{t('settings.accounts')}</h1>
        <button
          onClick={() => setShowForm(true)}
          className="flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          <Plus className="w-4 h-4 mr-2" />
          {t('common.add')}
        </button>
      </div>

      {saveSuccess && (
        <div className="mb-4 p-3 bg-green-50 border border-green-200 rounded-lg flex items-center text-green-700">
          <CheckCircle className="w-5 h-5 mr-2" />
          {t('common.saveSuccess')}
        </div>
      )}

      {(showForm || editingId) && (
        <div className="bg-white border border-gray-200 rounded-lg p-6 mb-6">
          <form onSubmit={handleSubmit}>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('account.email')}</label>
                <input
                  type="email"
                  required
                  value={formData.email}
                  onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('account.imapHost')}</label>
                <input
                  type="text"
                  required
                  value={formData.imapHost}
                  onChange={(e) => setFormData({ ...formData, imapHost: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                  placeholder="imap.example.com"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('account.imapPort')}</label>
                <input
                  type="number"
                  required
                  value={formData.imapPort}
                  onChange={(e) => setFormData({ ...formData, imapPort: parseInt(e.target.value) })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">{t('account.password')}</label>
                <input
                  type="password"
                  required={!editingId}
                  value={formData.password}
                  onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                />
              </div>
            </div>
            
            {testResult && (
              <div className={`mt-4 p-3 rounded-lg flex items-center ${testResult.success ? 'bg-green-50 text-green-700 border border-green-200' : 'bg-red-50 text-red-700 border border-red-200'}`}>
                {testResult.success ? <CheckCircle className="w-5 h-5 mr-2" /> : <XCircle className="w-5 h-5 mr-2" />}
                {testResult.message}
              </div>
            )}
            
            <div className="flex justify-between mt-6">
              <button
                type="button"
                onClick={handleTest}
                disabled={testing}
                className="px-4 py-2 text-blue-600 hover:text-blue-700 disabled:opacity-50 border border-blue-600 rounded-lg hover:bg-blue-50"
              >
                {testing ? t('account.testing') : t('common.test')}
              </button>
              <div className="space-x-3">
                <button
                  type="button"
                  onClick={handleCancel}
                  className="px-4 py-2 text-gray-600 hover:text-gray-800 border border-gray-300 rounded-lg hover:bg-gray-50"
                >
                  {t('common.cancel')}
                </button>
                <button
                  type="submit"
                  disabled={addMutation.isPending || updateMutation.isPending}
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                >
                  {addMutation.isPending || updateMutation.isPending ? t('common.saving') : t('common.save')}
                </button>
              </div>
            </div>
          </form>
        </div>
      )}

      <div className="space-y-3">
        {accounts.map((account) => (
          <div key={account.id} className="bg-white border border-gray-200 rounded-lg p-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center">
                <Mail className="w-5 h-5 text-gray-400 mr-3" />
                <div>
                  <p className="font-medium text-gray-900">{account.email}</p>
                  <p className="text-sm text-gray-500">{account.imapHost}:{account.imapPort}</p>
                </div>
              </div>
              <div className="flex space-x-2">
                <button
                  onClick={() => handleEdit(account)}
                  className="p-2 text-gray-400 hover:text-gray-600"
                >
                  <Edit className="w-4 h-4" />
                </button>
                <button
                  onClick={() => handleDeleteClick(account)}
                  className="p-2 text-gray-400 hover:text-red-600"
                >
                  <Trash2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      <ConfirmDialog
        isOpen={deleteConfirm.isOpen}
        title={t('confirm.title')}
        message={t('confirm.deleteAccount', { email: deleteConfirm.accountEmail })}
        confirmText={t('common.delete')}
        cancelText={t('common.cancel')}
        onConfirm={handleConfirmDelete}
        onCancel={() => setDeleteConfirm({ isOpen: false, accountId: null, accountEmail: '' })}
        isDanger
      />
    </div>
  );
};

export default Accounts;
