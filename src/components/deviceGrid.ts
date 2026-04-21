import type { AppState } from "../types/models";

import { renderDeviceCard } from "./deviceCard";

export function renderDeviceGrid(
  state: AppState,
  devices: AppState["devices"],
): string {
  if (state.bootstrapping) {
    return `
      <section class="empty-state">
        <strong>Loading devices...</strong>
        <p>Fetching devices from Tuya Cloud and preparing channel controls.</p>
      </section>
    `;
  }

  if (devices.length === 0) {
    return `
      <section class="empty-state">
        <strong>No devices to show</strong>
        <p>Adjust filters, refresh the dashboard, or review the saved Tuya connection.</p>
      </section>
    `;
  }

  return `
    <section class="device-grid device-grid--${state.uiPreferences.viewMode}">
      ${devices
        .map((device) =>
          renderDeviceCard(device, {
            viewMode: state.uiPreferences.viewMode,
            busyChannels: state.busyChannels,
            favoriteDeviceIds: state.uiPreferences.favoriteDeviceIds,
          }),
        )
        .join("")}
    </section>
  `;
}
