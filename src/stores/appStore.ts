import { desktopApi, type DesktopApi } from "../services/tauriApi";
import { filterDevices } from "../utils/deviceFilters";
import type {
  AppConfig,
  AppErrorPayload,
  AppState,
  BootstrapPayload,
  Device,
  DeviceChannel,
  SaveChannelAliasPayload,
  SaveDeviceAliasPayload,
  ToastMessage,
  UiPreferences,
} from "../types/models";

type Listener = (state: AppState) => void;
interface RefreshOptions {
  background?: boolean;
}

const DEFAULT_CONFIG: AppConfig = {
  clientId: "",
  clientSecret: "",
  baseUrl: "https://openapi.tuyaus.com",
  regionLabel: "Western America Data Center",
};

const DEFAULT_STATE: AppState = {
  bootstrapping: true,
  refreshing: false,
  testingConnection: false,
  savingConfig: false,
  configOpen: false,
  hasConfig: false,
  configDraft: DEFAULT_CONFIG,
  config: null,
  connection: {
    state: "needs_config",
    message: "Save your Tuya credentials to continue.",
  },
  devices: [],
  searchQuery: "",
  statusFilter: "all",
  uiPreferences: {
    viewMode: "user",
  },
  actionLog: [],
  busyChannels: {},
  toasts: [],
};

export class AppStore {
  private state: AppState = { ...DEFAULT_STATE };
  private readonly listeners = new Set<Listener>();
  private readonly api: DesktopApi;
  private refreshPromise: Promise<void> | null = null;

  constructor(api: DesktopApi = desktopApi) {
    this.api = api;
  }

  subscribe(listener: Listener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => {
      this.listeners.delete(listener);
    };
  }

  getState(): AppState {
    return this.state;
  }

  getVisibleDevices(): Device[] {
    return filterDevices(
      this.state.devices,
      this.state.searchQuery,
      this.state.statusFilter,
    );
  }

  async bootstrap(): Promise<void> {
    if (!this.api.isAvailable()) {
      this.patchState({
        ...DEFAULT_STATE,
        bootstrapping: false,
        configOpen: true,
        environmentMessage:
          "Run this interface through `npm run tauri dev` to access the Rust backend.",
      });
      return;
    }

    this.patchState({ bootstrapping: true });
    try {
      const payload = await this.api.loadBootstrap();
      this.applyBootstrap(payload);
    } catch (error) {
      this.patchState({
        bootstrapping: false,
        configOpen: true,
        connection: {
          state: "error",
          message: toMessage(error),
        },
      });
      this.pushToast({
        tone: "error",
        message: toMessage(error),
      });
    }
  }

  openConfig(): void {
    this.patchState({ configOpen: true });
  }

  closeConfig(): void {
    if (!this.state.hasConfig) {
      return;
    }
    this.patchState({ configOpen: false });
  }

  updateConfigDraft(field: keyof AppConfig, value: string): void {
    this.patchState({
      configDraft: {
        ...this.state.configDraft,
        [field]: value,
      },
    });
  }

  setSearchQuery(value: string): void {
    this.patchState({ searchQuery: value });
  }

  setStatusFilter(filter: AppState["statusFilter"]): void {
    this.patchState({ statusFilter: filter });
  }

  async setViewMode(viewMode: UiPreferences["viewMode"]): Promise<void> {
    const normalized = normalizeViewMode(viewMode);
    this.patchState({
      uiPreferences: {
        ...this.state.uiPreferences,
        viewMode: normalized,
      },
    });

    if (!this.api.isAvailable()) {
      return;
    }

    try {
      const preferences = await this.api.saveUiPreferences({
        viewMode: normalized,
      });
      this.patchState({
        uiPreferences: {
          viewMode: normalizeViewMode(preferences.viewMode),
        },
      });
    } catch (error) {
      this.pushToast({
        tone: "error",
        message: toMessage(error),
      });
    }
  }

  async testConnection(): Promise<void> {
    this.patchState({ testingConnection: true });
    try {
      const result = await this.api.testConnection(this.state.configDraft);
      this.patchState({
        testingConnection: false,
        connection: {
          state: "connected",
          message: `${result.message} ${result.deviceCount} device(s) visible.`,
          lastCheckedAt: Date.now(),
        },
      });
      this.pushToast({
        tone: "success",
        message: `Connection successful. ${result.deviceCount} device(s) visible.`,
      });
    } catch (error) {
      this.patchState({
        testingConnection: false,
        connection: {
          state: "error",
          message: toMessage(error),
          lastCheckedAt: Date.now(),
        },
      });
      this.pushToast({
        tone: "error",
        message: toMessage(error),
      });
    }
  }

  async saveConfig(): Promise<void> {
    this.patchState({ savingConfig: true });
    try {
      await this.api.saveConfig(this.state.configDraft);
      const masked = await this.api.getConfigMasked();
      this.patchState({
        savingConfig: false,
        hasConfig: true,
        config: masked,
        configDraft: {
          ...this.state.configDraft,
          clientSecret: "",
        },
        configOpen: false,
      });
      this.pushToast({
        tone: "success",
        message: "Configuration saved locally.",
      });
      await this.refreshDevices();
    } catch (error) {
      this.patchState({ savingConfig: false });
      this.pushToast({
        tone: "error",
        message: toMessage(error),
      });
    }
  }

  async refreshDevices(): Promise<void> {
    return this.refreshDevicesWithOptions();
  }

  private async refreshDevicesWithOptions(options: RefreshOptions = {}): Promise<void> {
    if (!this.api.isAvailable()) {
      return;
    }

    if (this.refreshPromise) {
      await this.refreshPromise;
      return;
    }

    const { background = false } = options;
    const hadDevices = this.state.devices.length > 0;
    this.patchState({ refreshing: true });

    this.refreshPromise = (async () => {
      try {
        const devices = await this.api.refreshAllDevices();
        this.patchState({
          bootstrapping: false,
          refreshing: false,
          hasConfig: true,
          devices,
          connection: {
            state: "connected",
            message: background
              ? "Device state updated from Tuya Cloud."
              : "Devices refreshed from Tuya Cloud.",
            lastCheckedAt: Date.now(),
          },
        });
      } catch (error) {
        const message = toMessage(error);
        const preserveVisibleDevices = background && hadDevices;
        this.patchState({
          bootstrapping: false,
          refreshing: false,
          connection: {
            state: "error",
            message: preserveVisibleDevices
              ? `${message} Showing cached devices.`
              : message,
            lastCheckedAt: Date.now(),
          },
        });

        if (!preserveVisibleDevices) {
          this.pushToast({
            tone: "error",
            message,
          });
        }
      } finally {
        this.refreshPromise = null;
      }
    })();

    await this.refreshPromise;
  }

  async toggleChannel(
    deviceId: string,
    channelCode: string,
    value: boolean,
  ): Promise<void> {
    const busyKey = `${deviceId}:${channelCode}`;
    const previousDevices = this.state.devices;

    this.patchState({
      devices: applyOptimisticChannelState(previousDevices, deviceId, channelCode, value),
      busyChannels: {
        ...this.state.busyChannels,
        [busyKey]: true,
      },
    });

    try {
      const result = await this.api.toggleChannel({
        deviceId,
        channelCode,
        value,
      });
      this.patchState({
        devices: applyStatusesToDevices(this.state.devices, result.deviceId, result.statuses),
        actionLog: [result.actionLogEntry, ...this.state.actionLog].slice(0, 50),
      });
    } catch (error) {
      this.patchState({
        devices: previousDevices,
      });
      this.pushToast({
        tone: "error",
        message: toMessage(error),
      });
    } finally {
      const busyChannels = { ...this.state.busyChannels };
      delete busyChannels[busyKey];
      this.patchState({ busyChannels });
    }
  }

  async saveDeviceAlias(payload: SaveDeviceAliasPayload): Promise<void> {
    try {
      await this.api.saveDeviceAlias(payload);
      this.patchState({
        devices: this.state.devices.map((device) =>
          device.id === payload.deviceId
            ? {
                ...device,
                name: payload.alias.trim() || resolveCloudName(device),
                metadata: {
                  alias: payload.alias.trim() || null,
                },
              }
            : device,
        ),
      });
      this.pushToast({
        tone: "success",
        message: "Device label updated.",
      });
    } catch (error) {
      this.pushToast({
        tone: "error",
        message: toMessage(error),
      });
    }
  }

  async saveChannelAlias(payload: SaveChannelAliasPayload): Promise<void> {
    try {
      await this.api.saveChannelAlias(payload);
      this.patchState({
        devices: this.state.devices.map((device) =>
          device.id === payload.deviceId
            ? {
                ...device,
                channels: device.channels.map((channel) =>
                  channel.code === payload.channelCode
                    ? {
                        ...channel,
                        alias: payload.alias.trim() || null,
                        displayName:
                          payload.alias.trim() ||
                          buildDefaultChannelName(channel),
                      }
                    : channel,
                ),
              }
            : device,
        ),
      });
      this.pushToast({
        tone: "success",
        message: "Channel label updated.",
      });
    } catch (error) {
      this.pushToast({
        tone: "error",
        message: toMessage(error),
      });
    }
  }

  removeToast(id: string): void {
    this.patchState({
      toasts: this.state.toasts.filter((toast) => toast.id !== id),
    });
  }

  private applyBootstrap(payload: BootstrapPayload): void {
    this.patchState({
      bootstrapping: false,
      hasConfig: payload.hasConfig,
      config: payload.config ?? null,
      configDraft: {
        clientId: payload.config?.clientId ?? DEFAULT_CONFIG.clientId,
        clientSecret: "",
        baseUrl: payload.config?.baseUrl ?? DEFAULT_CONFIG.baseUrl,
        regionLabel: payload.config?.regionLabel ?? DEFAULT_CONFIG.regionLabel,
      },
      configOpen: !payload.hasConfig,
      devices: payload.devices,
      uiPreferences: {
        viewMode: normalizeViewMode(payload.uiPreferences.viewMode),
      },
      actionLog: payload.actionLog,
      connection: payload.connection,
    });

    if (payload.hasConfig && payload.usesCachedDevices && this.api.isAvailable()) {
      void this.refreshDevicesWithOptions({ background: true });
    }
  }

  private patchState(patch: Partial<AppState>): void {
    const nextState = {
      ...this.state,
      ...patch,
    };

    if (!hasStateChanges(this.state, nextState, Object.keys(patch) as (keyof AppState)[])) {
      return;
    }

    this.state = nextState;
    for (const listener of this.listeners) {
      listener(this.state);
    }
  }

  private pushToast(input: Omit<ToastMessage, "id">): void {
    const id = crypto.randomUUID();
    this.patchState({
      toasts: [...this.state.toasts, { ...input, id }],
    });

    window.setTimeout(() => {
      this.removeToast(id);
    }, 2600);
  }
}

function applyOptimisticChannelState(
  devices: Device[],
  deviceId: string,
  channelCode: string,
  value: boolean,
): Device[] {
  return devices.map((device) =>
    device.id === deviceId
      ? {
          ...device,
          channels: device.channels.map((channel) =>
            channel.code === channelCode
              ? {
                  ...channel,
                  currentState: value,
                }
              : channel,
          ),
        }
      : device,
  );
}

function applyStatusesToDevices(
  devices: Device[],
  deviceId: string,
  statuses: { code: string; value: unknown }[],
): Device[] {
  return devices.map((device) => {
    if (device.id !== deviceId) {
      return device;
    }

    return {
      ...device,
      channels: device.channels.map((channel) => {
        const status = statuses.find((entry) => entry.code === channel.code);
        if (!status) {
          return channel;
        }

        return {
          ...channel,
          currentState:
            typeof status.value === "boolean"
              ? status.value
              : channel.currentState,
        };
      }),
    };
  });
}

function hasStateChanges(
  previous: AppState,
  next: AppState,
  keys: (keyof AppState)[],
): boolean {
  return keys.some((key) => previous[key] !== next[key]);
}

function resolveCloudName(device: Device): string {
  const summary = device.raw.summary as { name?: unknown } | undefined;
  const details = device.raw.details as { name?: unknown } | undefined;

  if (typeof summary?.name === "string" && summary.name.trim().length > 0) {
    return summary.name;
  }

  if (typeof details?.name === "string" && details.name.trim().length > 0) {
    return details.name;
  }

  return device.name;
}

function buildDefaultChannelName(channel: DeviceChannel): string {
  if (channel.code === "switch") {
    return "Main channel";
  }

  if (channel.code === "switch_led") {
    return "Backlight";
  }

  if (channel.code.startsWith("switch_")) {
    return `Switch ${channel.index}`;
  }

  return channel.displayName;
}

function normalizeViewMode(value: string | undefined): UiPreferences["viewMode"] {
  switch (value) {
    case "developer":
    case "detailed":
      return "developer";
    case "user":
    case "compact":
    default:
      return "user";
  }
}

function toMessage(error: unknown): string {
  if (error && typeof error === "object") {
    const payload = error as Partial<AppErrorPayload>;
    if (typeof payload.message === "string") {
      return payload.message;
    }
  }

  return "Unexpected application error.";
}
