export const PROTOCOL_VERSION = 'redclaw.v1' as const;
export const WS_SUBPROTOCOL = PROTOCOL_VERSION;

export const PUBLIC_HEALTH_PATH = '/health';
export const PAIR_PATH = '/pair';
export const EVENTS_STREAM_PATH = '/api/events';
export const WS_CHAT_PATH = '/ws/chat';

export const API_STATUS_PATH = '/api/status';
export const API_HEALTH_PATH = '/api/health';
export const API_CONFIG_PATH = '/api/config';
export const API_TOOLS_PATH = '/api/tools';
export const API_CRON_PATH = '/api/cron';
export const API_INTEGRATIONS_PATH = '/api/integrations';
export const API_DOCTOR_PATH = '/api/doctor';
export const API_MEMORY_PATH = '/api/memory';
export const API_COST_PATH = '/api/cost';
export const API_CLI_TOOLS_PATH = '/api/cli-tools';

export type ProtocolErrorCode =
  | 'validation_error'
  | 'auth_error'
  | 'policy_denied'
  | 'runtime_unavailable'
  | 'internal_error';

export interface ProtocolError {
  code: ProtocolErrorCode;
  message: string;
}

export interface ProtocolSessionHandle {
  session_id: string;
  version?: typeof PROTOCOL_VERSION;
}

export type ClientCommand =
  | { type: 'create_session' }
  | { type: 'resume_session'; handle: ProtocolSessionHandle }
  | { type: 'close_session'; handle: ProtocolSessionHandle }
  | {
      type: 'submit_turn';
      content: string;
      handle?: ProtocolSessionHandle;
    }
  | { type: 'message'; content: string };

export interface ClientEvent {
  type: string;
  content?: string;
  full_response?: string;
  message?: string;
  code?: ProtocolErrorCode;
  component?: string;
  provider?: string;
  model?: string;
  name?: string;
  tool?: string;
  args?: unknown;
  output?: string;
  success?: boolean;
  duration_ms?: number;
  timestamp?: string;
  tokens_used?: number;
  cost_usd?: number;
  [key: string]: unknown;
}

export function createMessageCommand(content: string): ClientCommand {
  return {
    type: 'message',
    content,
  };
}

export interface StatusResponse {
  provider: string | null;
  model: string;
  temperature: number;
  uptime_seconds: number;
  gateway_port: number;
  locale: string;
  memory_backend: string;
  paired: boolean;
  channels: Record<string, boolean>;
  health: HealthSnapshot;
}

export interface HealthSnapshot {
  pid: number;
  updated_at: string;
  uptime_seconds: number;
  components: Record<string, ComponentHealth>;
}

export interface ComponentHealth {
  status: string;
  updated_at: string;
  last_ok: string | null;
  last_error: string | null;
  restart_count: number;
}

export interface ToolSpec {
  name: string;
  description: string;
  parameters: any;
}

export interface CronJob {
  id: string;
  name: string | null;
  command: string;
  next_run: string;
  last_run: string | null;
  last_status: string | null;
  enabled: boolean;
}

export interface Integration {
  name: string;
  description: string;
  category: string;
  status: 'Available' | 'Active' | 'ComingSoon';
}

export interface DiagResult {
  severity: 'ok' | 'warn' | 'error';
  category: string;
  message: string;
}

export interface MemoryEntry {
  id: string;
  key: string;
  content: string;
  category: string;
  timestamp: string;
  session_id: string | null;
  score: number | null;
}

export interface CostSummary {
  session_cost_usd: number;
  daily_cost_usd: number;
  monthly_cost_usd: number;
  total_tokens: number;
  request_count: number;
  by_model: Record<string, ModelStats>;
}

export interface ModelStats {
  model: string;
  cost_usd: number;
  total_tokens: number;
  request_count: number;
}

export interface CliTool {
  name: string;
  path: string;
  version: string | null;
  category: string;
}

export type SSEEvent = ClientEvent;

export type WsMessage = ClientEvent;
