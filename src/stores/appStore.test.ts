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
    autoRefreshSeconds: 0,
    deviceOrder: [],
    favoriteDeviceIds: [],
  },
  actionLog: [],
  devices: [],
  connection: {
    state: "connected",
    message: "ok",
    lastCheckedAt: 123,
  },
  usesCachedDevices: false,
};

const cachedDevice = {
  id: "device-1",
  name: "Sala",
  online: true,
  category: "kg",
  productId: "abc",
  inferredType: "3-gang light switch",
  gangCount: 1,
  channels: [],
  raw: {
    summary: {},
    details: {},
    functions: [],
    status: [],
    capabilities: [],
    specifications: {},
  },
  metadata: null,
};

const deviceWithChannels = {
  ...cachedDevice,
  gangCount: 3,
  channels: [
    {
      code: "switch_1",
      displayName: "Channel 1",
      index: 1,
      currentState: false,
      controllable: true,
      alias: null,
    },
    {
      code: "switch_2",
      displayName: "Channel 2",
      index: 2,
      currentState: false,
      controllable: true,
      alias: null,
    },
    {
      code: "switch_3",
      displayName: "Channel 3",
      index: 3,
      currentState: false,
      controllable: true,
      alias: null,
    },
  ],
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
    refreshDeviceStatuses: () => Promise.resolve([]),
    toggleChannel: () => Promise.reject(new Error("not used in this test")),
    setDeviceChannels: () => Promise.reject(new Error("not used in this test")),
    saveDeviceAlias: () => Promise.resolve(),
    saveChannelAlias: () => Promise.resolve(),
    saveUiPreferences: ({ viewMode, autoRefreshSeconds, deviceOrder, favoriteDeviceIds }) =>
      Promise.resolve({
        viewMode: viewMode ?? bootstrapPayload.uiPreferences.viewMode,
        autoRefreshSeconds:
          autoRefreshSeconds ?? bootstrapPayload.uiPreferences.autoRefreshSeconds,
        deviceOrder: deviceOrder ?? bootstrapPayload.uiPreferences.deviceOrder,
        favoriteDeviceIds:
          favoriteDeviceIds ?? bootstrapPayload.uiPreferences.favoriteDeviceIds,
      }),
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

  it("saves auto refresh preference", async () => {
    const store = new AppStore(createApi());

    await store.setAutoRefreshSeconds(30);

    expect(store.getState().uiPreferences.autoRefreshSeconds).toBe(30);
  });

  it("reorders devices and persists the saved order", async () => {
    const saveUiPreferences = vi.fn().mockImplementation(({ deviceOrder }) =>
      Promise.resolve({
        ...bootstrapPayload.uiPreferences,
        deviceOrder: deviceOrder ?? [],
      }),
    );

    const store = new AppStore(
      createApi({
        loadBootstrap: () =>
          Promise.resolve({
            ...bootstrapPayload,
            devices: [
              { ...cachedDevice, id: "device-1", name: "Sala" },
              { ...cachedDevice, id: "device-2", name: "Cocina" },
            ],
          }),
        saveUiPreferences,
      }),
    );

    await store.bootstrap();
    await store.moveDevice("device-2", -1);

    expect(store.getState().devices.map((device) => device.id)).toEqual(["device-2", "device-1"]);
  });

  it("toggles favorite devices and persists the favorite ids", async () => {
    const saveUiPreferences = vi.fn().mockImplementation(({ favoriteDeviceIds }) =>
      Promise.resolve({
        ...bootstrapPayload.uiPreferences,
        favoriteDeviceIds: favoriteDeviceIds ?? [],
      }),
    );

    const store = new AppStore(
      createApi({
        loadBootstrap: () =>
          Promise.resolve({
            ...bootstrapPayload,
            devices: [
              { ...cachedDevice, id: "device-1", name: "Sala" },
              { ...cachedDevice, id: "device-2", name: "Cocina" },
            ],
          }),
        saveUiPreferences,
      }),
    );

    await store.bootstrap();
    await store.toggleFavoriteDevice("device-2");

    expect(store.getState().uiPreferences.favoriteDeviceIds).toEqual(["device-2"]);
    expect(store.getState().devices.map((device) => device.id)).toEqual(["device-2", "device-1"]);
    expect(saveUiPreferences).toHaveBeenCalledWith({
      favoriteDeviceIds: ["device-2"],
    });
  });

  it("refreshes in background after loading cached bootstrap devices", async () => {
    const refreshDeviceStatuses = vi.fn().mockResolvedValue([]);

    const store = new AppStore(
      createApi({
        loadBootstrap: () =>
          Promise.resolve({
            ...bootstrapPayload,
            usesCachedDevices: true,
            devices: [cachedDevice],
          }),
        refreshDeviceStatuses,
      }),
    );

    await store.bootstrap();
    await Promise.resolve();

    expect(refreshDeviceStatuses).toHaveBeenCalledTimes(1);
    expect(refreshDeviceStatuses).toHaveBeenCalledWith(["device-1"]);
  });

  it("updates all controllable channels for a device in one action", async () => {
    const setDeviceChannels = vi.fn().mockResolvedValue({
      deviceId: "device-1",
      statuses: [
        { code: "switch_1", value: true },
        { code: "switch_2", value: true },
        { code: "switch_3", value: true },
      ],
      actionLogEntry: {
        timestampMs: 123,
        action: "device_channels_on",
        deviceId: "device-1",
        deviceName: "Sala",
        channelCode: null,
        success: true,
        message: "3 channel(s) turned on",
      },
    });

    const store = new AppStore(
      createApi({
        loadBootstrap: () =>
          Promise.resolve({
            ...bootstrapPayload,
            devices: [deviceWithChannels],
          }),
        setDeviceChannels,
      }),
    );

    await store.bootstrap();
    await store.setDeviceChannels("device-1", true);

    expect(setDeviceChannels).toHaveBeenCalledWith({
      deviceId: "device-1",
      value: true,
    });
    expect(
      store
        .getState()
        .devices[0]
        ?.channels.map((channel) => channel.currentState),
    ).toEqual([true, true, true]);
  });
});
