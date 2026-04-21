import { invoke } from "@tauri-apps/api/core";

import type {
  ActionLogEntry,
  AppConfig,
  AppErrorPayload,
  BootstrapPayload,
  ConnectionTestResult,
  DeviceStatusUpdate,
  Device,
  MaskedAppConfig,
  SaveChannelAliasPayload,
  SaveDeviceAliasPayload,
  SaveUiPreferencesPayload,
  SetDeviceChannelsPayload,
  SetDeviceChannelsResult,
  ToggleChannelPayload,
  ToggleChannelResult,
  UiPreferences,
} from "../types/models";

export interface DesktopApi {
  isAvailable(): boolean;
  loadBootstrap(): Promise<BootstrapPayload>;
  getConfigMasked(): Promise<MaskedAppConfig | null>;
  saveConfig(payload: AppConfig): Promise<void>;
  testConnection(payload: AppConfig): Promise<ConnectionTestResult>;
  listDevices(): Promise<Device[]>;
  refreshAllDevices(): Promise<Device[]>;
  refreshDeviceStatuses(deviceIds: string[]): Promise<DeviceStatusUpdate[]>;
  toggleChannel(payload: ToggleChannelPayload): Promise<ToggleChannelResult>;
  setDeviceChannels(payload: SetDeviceChannelsPayload): Promise<SetDeviceChannelsResult>;
  saveDeviceAlias(payload: SaveDeviceAliasPayload): Promise<void>;
  saveChannelAlias(payload: SaveChannelAliasPayload): Promise<void>;
  saveUiPreferences(payload: SaveUiPreferencesPayload): Promise<UiPreferences>;
  getActionLog(): Promise<ActionLogEntry[]>;
}

class TauriDesktopApi implements DesktopApi {
  isAvailable(): boolean {
    return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
  }

  loadBootstrap(): Promise<BootstrapPayload> {
    return this.command<BootstrapPayload>("load_bootstrap");
  }

  getConfigMasked(): Promise<MaskedAppConfig | null> {
    return this.command<MaskedAppConfig | null>("get_config_masked");
  }

  saveConfig(payload: AppConfig): Promise<void> {
    return this.command("save_config", { payload });
  }

  testConnection(payload: AppConfig): Promise<ConnectionTestResult> {
    return this.command<ConnectionTestResult>("test_connection", { payload });
  }

  listDevices(): Promise<Device[]> {
    return this.command<Device[]>("list_devices");
  }

  refreshAllDevices(): Promise<Device[]> {
    return this.command<Device[]>("refresh_all_devices");
  }

  refreshDeviceStatuses(deviceIds: string[]): Promise<DeviceStatusUpdate[]> {
    return this.command<DeviceStatusUpdate[]>("refresh_device_statuses", { deviceIds });
  }

  toggleChannel(payload: ToggleChannelPayload): Promise<ToggleChannelResult> {
    return this.command<ToggleChannelResult>("toggle_channel", { payload });
  }

  setDeviceChannels(payload: SetDeviceChannelsPayload): Promise<SetDeviceChannelsResult> {
    return this.command<SetDeviceChannelsResult>("set_device_channels", { payload });
  }

  saveDeviceAlias(payload: SaveDeviceAliasPayload): Promise<void> {
    return this.command("save_device_alias", { payload });
  }

  saveChannelAlias(payload: SaveChannelAliasPayload): Promise<void> {
    return this.command("save_channel_alias", { payload });
  }

  saveUiPreferences(payload: SaveUiPreferencesPayload): Promise<UiPreferences> {
    return this.command<UiPreferences>("save_ui_preferences", { payload });
  }

  getActionLog(): Promise<ActionLogEntry[]> {
    return this.command<ActionLogEntry[]>("get_action_log");
  }

  private async command<T>(
    name: string,
    args?: Record<string, unknown>,
  ): Promise<T> {
    try {
      return await invoke<T>(name, args);
    } catch (error) {
      throw normalizeCommandError(error);
    }
  }
}

export const desktopApi: DesktopApi = new TauriDesktopApi();

export function normalizeCommandError(error: unknown): AppErrorPayload {
  if (typeof error === "string") {
    try {
      return JSON.parse(error) as AppErrorPayload;
    } catch {
      return {
        code: "unknown",
        message: error,
      };
    }
  }

  if (error && typeof error === "object") {
    const payload = error as Partial<AppErrorPayload>;
    if (typeof payload.message === "string") {
      return {
        code: typeof payload.code === "string" ? payload.code : "unknown",
        message: payload.message,
        details:
          typeof payload.details === "string" ? payload.details : undefined,
      };
    }
  }

  return {
    code: "unknown",
    message: "Unexpected command error.",
  };
}
