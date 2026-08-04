export type BetStatus = 'pending' | 'won' | 'lost';
export type Prediction = 'home_win' | 'away_win' | 'draw';

export interface Bet {
  id: string;
  user_id: string;
  group_id: string | null;
  event_id: string | null;
  prediction: Prediction | null;
  amount: number;
  odds: number;
  status: BetStatus;
  created_at: string;
  user_name: string;
  user_email: string;
  user_avatar_url: string | null;
  home_team: string | null;
  away_team: string | null;
}

export interface CreateBetRequest {
  group_id: string;
  event_id: string;
  prediction: Prediction;
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
  betted: number;
}

export interface MeResponse {
  user: PublicUser;
  groups: GroupWithBalance[];
}

export interface Event {
  id: string;
  external_id: string;
  home_team: string;
  away_team: string;
  championship: string;
  start_time: string;
  status: string;
  home_score: number | null;
  away_score: number | null;
  home_odds: number | null;
  draw_odds: number | null;
  away_odds: number | null;
  raw_data: unknown;
  created_at: string;
}
