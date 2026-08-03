import { useState } from 'react';
import { useAuth } from '../context/AuthContext';
import { createGroup, getInviteCode, joinGroup } from '../api/client';
import type { Group } from '../types';

interface Props {
  selectedGroupId: string | null;
  onSelect: (id: string | null) => void;
}

export default function GroupSwitcher({ selectedGroupId, onSelect }: Props) {
  const { groups, addGroup } = useAuth();
  const [showCreate, setShowCreate] = useState(false);
  const [showJoin, setShowJoin] = useState(false);
  const [newName, setNewName] = useState('');
  const [inviteCode, setInviteCode] = useState('');
  const [inviteLink, setInviteLink] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const selected = groups.find((g) => g.id === selectedGroupId);

  const handleSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const val = e.target.value;
    if (val === '__create__') {
      setShowCreate(true);
      setShowJoin(false);
    } else if (val === '__join__') {
      setShowJoin(true);
      setShowCreate(false);
      setInviteLink(null);
    } else if (val === '') {
      onSelect(null);
    } else {
      onSelect(val);
      setShowCreate(false);
      setShowJoin(false);
      setInviteLink(null);
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
    } catch {
      alert('Failed to create group');
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
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to join');
    } finally {
      setLoading(false);
    }
  };

  const handleGetInvite = async () => {
    if (!selectedGroupId) return;
    setLoading(true);
    try {
      const data = (await getInviteCode(selectedGroupId)) as { invite_code: string };
      setInviteLink(`${window.location.origin}?join=${data.invite_code}`);
    } catch {
      alert('Failed to get invite link');
    } finally {
      setLoading(false);
    }
  };

  if (groups.length === 0 && !showCreate && !showJoin) {
    return (
      <div className="group-switcher">
        <select value="" onChange={handleSelect} className="group-select">
          <option value="">No groups</option>
          <option value="__create__">+ Create group</option>
          <option value="__join__">+ Join by code</option>
        </select>
      </div>
    );
  }

  return (
    <div className="group-switcher">
      <select
        value={selectedGroupId || ''}
        onChange={handleSelect}
        className="group-select"
      >
        <option value="">Select group...</option>
        {groups.map((g) => (
          <option key={g.id} value={g.id}>
            {g.name} ({g.balance.toFixed(0)} pts)
          </option>
        ))}
        <option disabled>──</option>
        <option value="__create__">+ Create group</option>
        <option value="__join__">+ Join by code</option>
      </select>

      {selected && selectedGroupId && (
        <button onClick={handleGetInvite} className="btn-invite" disabled={loading}>
          Get invite link
        </button>
      )}

      {showCreate && (
        <form onSubmit={handleCreate} className="inline-form">
          <input
            placeholder="Group name"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            required
          />
          <button type="submit" disabled={loading}>Create</button>
        </form>
      )}

      {showJoin && (
        <form onSubmit={handleJoin} className="inline-form">
          <input
            placeholder="Invite code"
            value={inviteCode}
            onChange={(e) => setInviteCode(e.target.value)}
            required
          />
          <button type="submit" disabled={loading}>Join</button>
        </form>
      )}

      {inviteLink && (
        <div className="invite-link-bar">
          <code>{inviteLink}</code>
          <button onClick={() => navigator.clipboard.writeText(inviteLink)}>Copy</button>
        </div>
      )}
    </div>
  );
}
