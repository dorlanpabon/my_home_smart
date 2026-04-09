import type { DesktopApi } from "../services/tauriApi";
import type { BootstrapPayload } from "../types/models";

import { AppStore } from "./appStore";

const bootstrapPayload: BootstrapPayload = {
  hasConfig: true,
  config: {
    clientId: "abc",
    clientSecretMasked: "****1234",
    clientSecretPresent: true,
    baseUrl: "https://openapi.tuyaus.com",
    regionLabel: "Western America Data Center",
  },
  uiPreferences: {
    viewMode: "developer",
  },
  actionLog: [],
  devices: [],
  connection: {
    state: "connected",
    message: "ok",
    lastCheckedAt: 123,
  },
};

function createApi(overrides: Partial<DesktopApi> = {}): DesktopApi {
  return {
    isAvailable: () => true,
    loadBootstrap: () => Promise.resolve(bootstrapPayload),
    getConfigMasked: () => Promise.resolve(bootstrapPayload.config ?? null),
    saveConfig: () => Promise.resolve(),
    testConnection: () =>
      Promise.resolve({
        success: true,
        message: "ok",
        baseUrl: bootstrapPayload.config!.baseUrl,
        regionLabel: bootstrapPayload.config!.regionLabel,
        deviceCount: 1,
      }),
    listDevices: () => Promise.resolve([]),
    refreshAllDevices: () => Promise.resolve([]),
    toggleChannel: () => Promise.reject(new Error("not used in this test")),
    saveDeviceAlias: () => Promise.resolve(),
    saveChannelAlias: () => Promise.resolve(),
    saveUiPreferences: ({ viewMode }) => Promise.resolve({ viewMode }),
    getActionLog: () => Promise.resolve([]),
    ...overrides,
  };
}

describe("AppStore", () => {
  it("hydrates bootstrap payload into state", async () => {
    const store = new AppStore(createApi());

    await store.bootstrap();

    expect(store.getState().bootstrapping).toBe(false);
    expect(store.getState().hasConfig).toBe(true);
    expect(store.getState().uiPreferences.viewMode).toBe("developer");
  });

  it("updates config draft fields", () => {
    const store = new AppStore(createApi());

    store.updateConfigDraft("clientId", "new-id");

    expect(store.getState().configDraft.clientId).toBe("new-id");
  });
});
