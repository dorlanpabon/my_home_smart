export interface AppConfig {
  clientId: string;
  clientSecret: string;
  baseUrl: string;
  regionLabel: string;
}

export interface MaskedAppConfig {
  clientId: string;
  clientSecretMasked: string;
  clientSecretPresent: boolean;
  baseUrl: string;
  regionLabel: string;
}

export interface DeviceAlias {
  deviceId: string;
  alias: string;
}

export interface ChannelAlias {
  deviceId: string;
  channelCode: string;
  alias: string;
}

export interface UiPreferences {
  viewMode: "user" | "developer";
  autoRefreshSeconds: 0 | 15 | 30 | 60;
  deviceOrder: string[];
  favoriteDeviceIds: string[];
}

export interface DeviceLocalMetadata {
  alias?: string | null;
}

export interface TuyaFunction {
  code: string;
  valueType?: string | null;
  values?: unknown;
  mode?: string | null;
  support?: string | null;
  name?: string | null;
  description?: string | null;
}

export interface TuyaStatus {
  code: string;
  value: unknown;
}

export interface RawDeviceData {
  summary: unknown;
  details: unknown;
  functions: TuyaFunction[];
  status: TuyaStatus[];
  capabilities: TuyaFunction[];
  specifications: unknown;
}

export interface DeviceChannel {
  code: string;
  displayName: string;
  index: number;
  currentState: boolean | null;
  controllable: boolean;
  alias?: string | null;
}

export interface Device {
  id: string;
  name: string;
  online: boolean;
  category?: string | null;
  productId?: string | null;
  inferredType: string;
  gangCount: number;
  channels: DeviceChannel[];
  raw: RawDeviceData;
  metadata?: DeviceLocalMetadata | null;
}

export interface ConnectionStatus {
  state: "needs_config" | "connected" | "error";
  message?: string | null;
  lastCheckedAt?: number | null;
}

export interface ConnectionTestResult {
  success: boolean;
  message: string;
  baseUrl: string;
  regionLabel: string;
  deviceCount: number;
}

export interface ActionLogEntry {
  timestampMs: number;
  action: string;
  deviceId?: string | null;
  deviceName?: string | null;
  channelCode?: string | null;
  success: boolean;
  message: string;
}

export interface BootstrapPayload {
  hasConfig: boolean;
  config?: MaskedAppConfig | null;
  uiPreferences: UiPreferences;
  actionLog: ActionLogEntry[];
  devices: Device[];
  connection: ConnectionStatus;
  usesCachedDevices?: boolean;
}

export interface ToggleChannelPayload {
  deviceId: string;
  channelCode: string;
  value: boolean;
}

export interface SetDeviceChannelsPayload {
  deviceId: string;
  value: boolean;
}

export interface ToggleChannelResult {
  deviceId: string;
  statuses: TuyaStatus[];
  actionLogEntry: ActionLogEntry;
}

export interface SetDeviceChannelsResult {
  deviceId: string;
  statuses: TuyaStatus[];
  actionLogEntry: ActionLogEntry;
}

export interface DeviceStatusUpdate {
  deviceId: string;
  statuses: TuyaStatus[];
}

export interface SaveDeviceAliasPayload {
  deviceId: string;
  alias: string;
}

export interface SaveChannelAliasPayload {
  deviceId: string;
  channelCode: string;
  alias: string;
}

export interface SaveUiPreferencesPayload {
  viewMode?: UiPreferences["viewMode"];
  autoRefreshSeconds?: UiPreferences["autoRefreshSeconds"];
  deviceOrder?: UiPreferences["deviceOrder"];
  favoriteDeviceIds?: UiPreferences["favoriteDeviceIds"];
}

export interface AppErrorPayload {
  code: string;
  message: string;
  details?: string | null;
}

export interface ToastMessage {
  id: string;
  tone: "info" | "success" | "error";
  message: string;
}

export interface AppState {
  bootstrapping: boolean;
  refreshing: boolean;
  testingConnection: boolean;
  savingConfig: boolean;
  configOpen: boolean;
  hasConfig: boolean;
  configDraft: AppConfig;
  config: MaskedAppConfig | null;
  connection: ConnectionStatus;
  devices: Device[];
  searchQuery: string;
  statusFilter: "all" | "online" | "offline" | "favorites";
  uiPreferences: UiPreferences;
  actionLog: ActionLogEntry[];
  busyChannels: Record<string, boolean>;
  toasts: ToastMessage[];
  environmentMessage?: string;
}
