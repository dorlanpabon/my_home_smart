import type { Device, DeviceChannel, UiPreferences } from "../types/models";
import { escapeHtml } from "../utils/escape";
import { formatDeviceSubtitle } from "../utils/format";
import {
  renderBulbIcon,
  renderFavoriteIcon,
  renderPowerIcon,
  renderSwitchIcon,
} from "./icons";

interface DeviceCardOptions {
  viewMode: UiPreferences["viewMode"];
  busyChannels: Record<string, boolean>;
  favoriteDeviceIds: string[];
}

export function renderDeviceCard(
  device: Device,
  options: DeviceCardOptions,
): string {
  const favorite = options.favoriteDeviceIds.includes(device.id);
  return options.viewMode === "user"
    ? renderUserDeviceCard(device, options.busyChannels, favorite)
    : renderDeveloperDeviceCard(device, options.busyChannels, favorite);
}

function renderUserDeviceCard(
  device: Device,
  busyChannels: Record<string, boolean>,
  favorite: boolean,
): string {
  const bulkActions = renderBulkActions(device, busyChannels);
  return `
    <article class="device-card device-card--user ${device.online ? "is-online" : "is-offline"}">
      <div class="device-card__user-head">
        <div class="device-card__badge">${renderSwitchIcon()}</div>
        <div class="device-card__user-copy">
          <h3>${escapeHtml(device.name)}</h3>
          <p>${device.gangCount} ch</p>
        </div>
        <span class="device-card__presence ${device.online ? "is-online" : "is-offline"}">
          ${device.online ? "Online" : "Offline"}
        </span>
        <button
          class="icon-button icon-button--square device-card__favorite ${favorite ? "is-active" : ""}"
          data-action="toggle-favorite-device"
          data-device-id="${escapeHtml(device.id)}"
          title="${favorite ? "Remove favorite" : "Add favorite"}"
          aria-label="${favorite ? "Remove favorite" : "Add favorite"}"
        >
          ${renderFavoriteIcon(favorite)}
        </button>
      </div>

      ${bulkActions}

      <div class="channel-tile-grid">
        ${
          device.channels.length > 0
            ? device.channels
                .map((channel) => renderChannelTile(device, channel, busyChannels))
                .join("")
            : `<div class="empty-state empty-state--inline">No switch channels detected.</div>`
        }
      </div>
    </article>
  `;
}

function renderDeveloperDeviceCard(
  device: Device,
  busyChannels: Record<string, boolean>,
  favorite: boolean,
): string {
  const bulkActions = renderBulkActions(device, busyChannels);
  return `
    <article class="device-card device-card--developer ${device.online ? "is-online" : "is-offline"}">
      <div class="device-card__header">
        <div class="device-card__title">
          <div class="device-card__badge device-card__badge--small">${renderSwitchIcon()}</div>
          <div>
            <div class="device-card__title-row">
              <h3>${escapeHtml(device.name)}</h3>
              <span class="status-badge ${device.online ? "status-badge--online" : "status-badge--offline"}">
                ${device.online ? "Online" : "Offline"}
              </span>
            </div>
            <p class="device-card__subtitle">${escapeHtml(formatDeviceSubtitle(device))}</p>
          </div>
        </div>
        <div class="device-card__actions">
          <button
            class="icon-button icon-button--square device-card__favorite ${favorite ? "is-active" : ""}"
            data-action="toggle-favorite-device"
            data-device-id="${escapeHtml(device.id)}"
            title="${favorite ? "Remove favorite" : "Add favorite"}"
            aria-label="${favorite ? "Remove favorite" : "Add favorite"}"
          >
            ${renderFavoriteIcon(favorite)}
          </button>
          <button
            class="icon-button icon-button--square"
            data-action="move-device-up"
            data-device-id="${escapeHtml(device.id)}"
            title="Move device up"
          >
            ↑
          </button>
          <button
            class="icon-button icon-button--square"
            data-action="move-device-down"
            data-device-id="${escapeHtml(device.id)}"
            title="Move device down"
          >
            ↓
          </button>
          <button
            class="icon-button"
            data-action="copy-device-id"
            data-device-id="${escapeHtml(device.id)}"
            title="Copy device id"
          >
            Copy id
          </button>
        </div>
      </div>

      <div class="device-card__meta">
        <span><strong>Id</strong> ${escapeHtml(device.id)}</span>
        <span><strong>Channels</strong> ${device.gangCount || 0}</span>
        <span><strong>Product</strong> ${escapeHtml(device.productId ?? "n/a")}</span>
      </div>

      ${bulkActions}

      <div class="channel-strip">
        ${device.channels
          .map((channel) => renderDeveloperChannelRow(device, channel, busyChannels))
          .join("")}
      </div>

      ${renderAliasEditors(device)}
    </article>
  `;
}

function renderChannelTile(
  device: Device,
  channel: DeviceChannel,
  busyChannels: Record<string, boolean>,
): string {
  const busyKey = `${device.id}:${channel.code}`;
  const isBusy = Boolean(busyChannels[busyKey]);
  const active = channel.currentState === true;
  const unknown = channel.currentState === null;
  const disabled = isBusy || !device.online || !channel.controllable;

  return `
    <button
      class="channel-tile ${active ? "is-on" : "is-off"} ${unknown ? "is-unknown" : ""}"
      data-action="toggle-channel"
      data-device-id="${escapeHtml(device.id)}"
      data-channel-code="${escapeHtml(channel.code)}"
      data-value="${active ? "false" : "true"}"
      ${disabled ? "disabled" : ""}
      title="${escapeHtml(channel.displayName)}"
    >
      <span class="channel-tile__icon">${renderBulbIcon(active)}</span>
      <span class="channel-tile__label">${escapeHtml(channel.displayName)}</span>
      <span class="channel-tile__state">
        ${isBusy ? "..." : unknown ? "?" : active ? "ON" : "OFF"}
      </span>
    </button>
  `;
}

function renderDeveloperChannelRow(
  device: Device,
  channel: DeviceChannel,
  busyChannels: Record<string, boolean>,
): string {
  const busyKey = `${device.id}:${channel.code}`;
  const isBusy = Boolean(busyChannels[busyKey]);
  const active = channel.currentState === true;
  const unknown = channel.currentState === null;
  const disabled = isBusy || !device.online || !channel.controllable;

  return `
    <section class="channel-row ${active ? "is-on" : ""}">
      <div class="channel-row__identity">
        <span class="channel-row__glyph">${renderPowerIcon(active)}</span>
        <div>
          <strong>${escapeHtml(channel.displayName)}</strong>
          <p>${escapeHtml(channel.code)}${channel.controllable ? "" : " - read only"}</p>
        </div>
      </div>
      <button
        class="toggle-button ${active ? "is-active" : ""}"
        data-action="toggle-channel"
        data-device-id="${escapeHtml(device.id)}"
        data-channel-code="${escapeHtml(channel.code)}"
        data-value="${active ? "false" : "true"}"
        ${disabled ? "disabled" : ""}
      >
        ${isBusy ? "Sending..." : unknown ? "Set state" : active ? "Turn off" : "Turn on"}
      </button>
    </section>
  `;
}

function renderBulkActions(
  device: Device,
  busyChannels: Record<string, boolean>,
): string {
  const controllableChannels = device.channels.filter((channel) => channel.controllable);
  if (controllableChannels.length < 2) {
    return "";
  }

  const deviceBusy = controllableChannels.some(
    (channel) => busyChannels[`${device.id}:${channel.code}`],
  );
  const disabled = deviceBusy || !device.online;

  return `
    <div class="device-card__bulk-actions">
      <button
        class="button button--ghost device-card__bulk-button"
        data-action="set-device-channels"
        data-device-id="${escapeHtml(device.id)}"
        data-value="true"
        ${disabled ? "disabled" : ""}
      >
        ${deviceBusy ? "Sending..." : "All on"}
      </button>
      <button
        class="button button--ghost device-card__bulk-button"
        data-action="set-device-channels"
        data-device-id="${escapeHtml(device.id)}"
        data-value="false"
        ${disabled ? "disabled" : ""}
      >
        ${deviceBusy ? "Sending..." : "All off"}
      </button>
    </div>
  `;
}

function renderAliasEditors(device: Device): string {
  return `
    <div class="device-card__editor">
      <form class="alias-form" data-form="device-alias">
        <input type="hidden" name="deviceId" value="${escapeHtml(device.id)}" />
        <label>
          <span>Local device label</span>
          <div class="alias-form__row">
            <input
              type="text"
              name="alias"
              value="${escapeHtml(device.metadata?.alias ?? "")}"
              placeholder="Optional alias"
            />
            <button class="button button--ghost" type="submit">Save</button>
          </div>
        </label>
      </form>
      <div class="channel-alias-list">
        ${device.channels
          .map(
            (channel) => `
              <form class="alias-form alias-form--channel" data-form="channel-alias">
                <input type="hidden" name="deviceId" value="${escapeHtml(device.id)}" />
                <input type="hidden" name="channelCode" value="${escapeHtml(channel.code)}" />
                <label>
                  <span>${escapeHtml(channel.code)}</span>
                  <div class="alias-form__row">
                    <input type="text" name="alias" value="${escapeHtml(channel.alias ?? "")}" placeholder="Optional local label" />
                    <button class="button button--ghost" type="submit">Save</button>
                  </div>
                </label>
              </form>
            `,
          )
          .join("")}
      </div>
    </div>
  `;
}
