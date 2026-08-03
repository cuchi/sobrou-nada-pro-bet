export type BetStatus = 'pending' | 'won' | 'lost';

export interface Bet {
  id: string;
  user_id: string;
  group_id: string | null;
  amount: number;
  odds: number;
  status: BetStatus;
  created_at: string;
  user_name: string;
  user_email: string;
}

export interface CreateBetRequest {
  group_id: string;
  amount: number;
  odds: number;
}

export interface PublicUser {
  id: string;
  name: string;
  email: string;
  avatar_url: string | null;
}

export interface AuthResponse {
  token: string;
  user: PublicUser;
}

export interface Group {
  id: string;
  name: string;
  invite_code: string;
  owner_id: string;
  created_at: string;
}

export interface GroupWithBalance {
  id: string;
  name: string;
  invite_code: string;
  owner_id: string;
  created_at: string;
  balance: number;
}

export interface LeaderboardEntry {
  user_id: string;
  name: string;
  email: string;
  avatar_url: string | null;
  balance: number;
}

export interface MeResponse {
  user: PublicUser;
  groups: GroupWithBalance[];
}
