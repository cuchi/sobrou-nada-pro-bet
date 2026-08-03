export type BetStatus = 'pending' | 'won' | 'lost';

export interface Bet {
  id: string;
  user_id: string;
  amount: number;
  odds: number;
  status: BetStatus;
  created_at: string;
}

export interface CreateBetRequest {
  amount: number;
  odds: number;
}

export interface PublicUser {
  id: string;
  name: string;
  email: string;
  balance: number;
  avatar_url: string | null;
}

export interface AuthResponse {
  token: string;
  user: PublicUser;
}
