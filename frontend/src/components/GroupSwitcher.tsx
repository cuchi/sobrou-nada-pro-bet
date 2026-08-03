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
    if (val === '') {
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
      setShowJoin(false);
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
      setShowCreate(false);
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

  return (
    <div className="group-switcher">
      <div className="group-bar">
        <select
          value={selectedGroupId || ''}
          onChange={handleSelect}
          className="group-select"
        >
          <option value="">
            {groups.length === 0 ? 'No groups' : 'Select group...'}
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
          + Create
        </button>
        <button
          onClick={() => { setShowJoin(true); setShowCreate(false); setInviteLink(null); }}
          className="btn-group-action"
        >
          + Join
        </button>

        {selected && selectedGroupId && (
          <button onClick={handleGetInvite} className="btn-invite" disabled={loading}>
            Invite
          </button>
        )}
      </div>

      {showCreate && (
        <form onSubmit={handleCreate} className="inline-form">
          <input
            placeholder="Group name"
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            required
          />
          <button type="submit" disabled={loading}>Create</button>
          <button type="button" className="btn-cancel" onClick={() => setShowCreate(false)}>
            Cancel
          </button>
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
          <button type="button" className="btn-cancel" onClick={() => setShowJoin(false)}>
            Cancel
          </button>
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
