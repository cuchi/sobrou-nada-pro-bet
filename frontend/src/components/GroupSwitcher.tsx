import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useAuth } from '../context/AuthContext';
import { createGroup, getInviteCode, joinGroup } from '../api/client';
import { useToast } from './Toast';
import type { Group } from '../types';

interface Props {
  selectedGroupId: string | null;
  onSelect: (id: string | null) => void;
}

export default function GroupSwitcher({ selectedGroupId, onSelect }: Props) {
  const { t } = useTranslation();
  const { groups, addGroup } = useAuth();
  const { toast } = useToast();
  const [showCreate, setShowCreate] = useState(false);
  const [showJoin, setShowJoin] = useState(false);
  const [newName, setNewName] = useState('');
  const [inviteCode, setInviteCode] = useState('');
  const [groupInvite, setGroupInvite] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const selected = groups.find((g) => g.id === selectedGroupId);

  const handleSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const val = e.target.value;
    if (val === '') {
      onSelect(null);
    } else {
      onSelect(val);
      setShowCreate(false);
      setShowJoin(false);
      setGroupInvite(null);
    }
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim()) return;
    setLoading(true);
    try {
      const g = (await createGroup(newName.trim())) as Group;
      const gw = { ...g, balance: 1000 };
      addGroup(gw);
      onSelect(g.id);
      setNewName('');
      setShowCreate(false);
      setShowJoin(false);
    } catch {
      toast(t('groupSwitcher.errors.createFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleJoin = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!inviteCode.trim()) return;
    setLoading(true);
    try {
      const data = (await joinGroup(inviteCode.trim())) as { group: Group };
      const gw = { ...data.group, balance: 1000 };
      addGroup(gw);
      onSelect(data.group.id);
      setInviteCode('');
      setShowJoin(false);
      setShowCreate(false);
    } catch (err) {
      toast(err instanceof Error ? err.message : t('groupSwitcher.errors.joinFailed'));
    } finally {
      setLoading(false);
    }
  };

  const handleGetInvite = async () => {
    if (!selectedGroupId) return;
    setLoading(true);
    try {
      const data = (await getInviteCode(selectedGroupId)) as { invite_code: string };
      setGroupInvite(data.invite_code);
    } catch {
      toast(t('groupSwitcher.errors.inviteFailed'));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="group-switcher">
      <div className="group-bar">
        <select
          value={selectedGroupId || ''}
          onChange={handleSelect}
          className="group-select"
        >
          <option value="">
            {groups.length === 0 ? t('groupSwitcher.noGroups') : t('groupSwitcher.selectGroup')}
          </option>
          {groups.map((g) => (
            <option key={g.id} value={g.id}>
              {g.name} ({g.balance.toFixed(0)} pts)
            </option>
          ))}
        </select>

        <button
          onClick={() => { setShowCreate(true); setShowJoin(false); }}
          className="btn-group-action"
        >
          {t('groupSwitcher.create')}
        </button>
        <button
          onClick={() => { setShowJoin(true); setShowCreate(false); setGroupInvite(null); }}
          className="btn-group-action"
        >
          {t('groupSwitcher.join')}
        </button>

        {selected && selectedGroupId && (
          <button onClick={handleGetInvite} className="btn-invite" disabled={loading}>
            {t('groupSwitcher.invite')}
          </button>
        )}
      </div>

      {showCreate && (
        <form onSubmit={handleCreate} className="inline-form">
          <input
            placeholder={t('groupSwitcher.groupNamePlaceholder')}
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            required
          />
          <button type="submit" disabled={loading}>{t('groupSwitcher.submitCreate')}</button>
          <button type="button" className="btn-cancel" onClick={() => setShowCreate(false)}>
            {t('groupSwitcher.cancel')}
          </button>
        </form>
      )}

      {showJoin && (
        <form onSubmit={handleJoin} className="inline-form">
          <input
            placeholder={t('groupSwitcher.inviteCodePlaceholder')}
            value={inviteCode}
            onChange={(e) => setInviteCode(e.target.value)}
            required
          />
          <button type="submit" disabled={loading}>{t('groupSwitcher.submitJoin')}</button>
          <button type="button" className="btn-cancel" onClick={() => setShowJoin(false)}>
            {t('groupSwitcher.cancel')}
          </button>
        </form>
      )}

      {groupInvite && (
        <div className="invite-code-bar">
          <code>{groupInvite}</code>
          <button onClick={() => navigator.clipboard.writeText(groupInvite)}>{t('groupSwitcher.copy')}</button>
          <button
            className="btn-invite-close"
            onClick={() => setGroupInvite(null)}
            aria-label={t('groupSwitcher.closeAriaLabel')}
          >
            ×
          </button>
        </div>
      )}
    </div>
  );
}
