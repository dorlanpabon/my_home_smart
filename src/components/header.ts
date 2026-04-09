import type { AppState } from "../types/models";
import { escapeHtml } from "../utils/escape";
import { formatConnectionState, formatLastSeen } from "../utils/format";
import { renderRefreshIcon, renderSettingsIcon } from "./icons";

export function renderHeader(state: AppState, visibleCount: number): string {
  return state.uiPreferences.viewMode === "user"
    ? renderUserHeader(state, visibleCount)
    : renderDeveloperHeader(state, visibleCount);
}

function renderUserHeader(state: AppState, visibleCount: number): string {
  return `
    <header class="topbar topbar--user">
      <div class="topbar__compact-row">
        <div class="topbar__compact-title">
          <div>
            <p class="eyebrow">Tuya Desk</p>
            <h1>Lights</h1>
          </div>
          <div class="connection-mini connection-mini--${escapeHtml(state.connection.state)}">
            <span class="connection-mini__dot"></span>
            <span>${escapeHtml(formatConnectionState(state.connection.state))}</span>
          </div>
        </div>

        <div class="topbar__compact-actions">
          <div class="segmented segmented--compact" role="tablist" aria-label="Experience mode">
            ${renderModeButton(state, "user", "User")}
            ${renderModeButton(state, "developer", "Dev")}
          </div>
          <button
            class="icon-button icon-button--square"
            data-action="refresh-devices"
            ${state.refreshing ? "disabled" : ""}
            title="Refresh devices"
            aria-label="Refresh devices"
          >
            ${renderRefreshIcon()}
          </button>
          <button
            class="icon-button icon-button--square"
            data-action="open-config"
            title="Open settings"
            aria-label="Open settings"
          >
            ${renderSettingsIcon()}
          </button>
        </div>
      </div>

      <div class="topbar__compact-row topbar__compact-row--lower">
        <label class="search search--compact">
          <input
            type="search"
            name="searchQuery"
            placeholder="Search"
            value="${escapeHtml(state.searchQuery)}"
          />
        </label>

        <div class="segmented segmented--compact" role="tablist" aria-label="Status filter">
          ${renderFilterButton(state, "all", "All")}
          ${renderFilterButton(state, "online", "On")}
          ${renderFilterButton(state, "offline", "Off")}
        </div>

        <span class="topbar__mini-stats">${visibleCount}/${state.devices.length} visible</span>
      </div>
    </header>
  `;
}

function renderDeveloperHeader(state: AppState, visibleCount: number): string {
  return `
    <header class="topbar topbar--developer">
      <div class="topbar__identity">
        <p class="eyebrow">Developer console</p>
        <h1>Tuya Desk</h1>
        <p class="subtle">Full device diagnostics, aliases, ids and channel controls.</p>
      </div>

      <div class="topbar__controls">
        <div class="connection-pill connection-pill--${escapeHtml(state.connection.state)}">
          <span class="connection-pill__dot"></span>
          <div>
            <strong>${escapeHtml(formatConnectionState(state.connection.state))}</strong>
            <span>${escapeHtml(state.connection.message ?? "No status message.")}</span>
          </div>
        </div>

        <div class="toolbar">
          <label class="search">
            <span>Search</span>
            <input
              type="search"
              name="searchQuery"
              placeholder="Search device or channel"
              value="${escapeHtml(state.searchQuery)}"
            />
          </label>

          <div class="segmented" role="tablist" aria-label="Experience mode">
            ${renderModeButton(state, "user", "User")}
            ${renderModeButton(state, "developer", "Developer")}
          </div>

          <div class="segmented" role="tablist" aria-label="Status filter">
            ${renderFilterButton(state, "all", "All")}
            ${renderFilterButton(state, "online", "Online")}
            ${renderFilterButton(state, "offline", "Offline")}
          </div>

          <button class="button button--secondary" data-action="open-config">
            Settings
          </button>
          <button
            class="button button--primary"
            data-action="refresh-devices"
            ${state.refreshing ? "disabled" : ""}
          >
            ${state.refreshing ? "Refreshing..." : "Refresh"}
          </button>
        </div>

        <div class="topbar__stats">
          <span>${visibleCount} visible</span>
          <span>${state.devices.length} devices</span>
          <span>Last sync ${escapeHtml(formatLastSeen(state.connection.lastCheckedAt))}</span>
        </div>
      </div>
    </header>
  `;
}

function renderModeButton(
  state: AppState,
  viewMode: AppState["uiPreferences"]["viewMode"],
  label: string,
): string {
  const active = state.uiPreferences.viewMode === viewMode;
  return `
    <button
      class="segmented__button ${active ? "is-active" : ""}"
      data-action="set-view-mode"
      data-view-mode="${viewMode}"
      aria-pressed="${active}"
    >
      ${label}
    </button>
  `;
}

function renderFilterButton(
  state: AppState,
  filter: AppState["statusFilter"],
  label: string,
): string {
  const active = state.statusFilter === filter;
  return `
    <button
      class="segmented__button ${active ? "is-active" : ""}"
      data-action="set-filter"
      data-filter="${filter}"
      aria-pressed="${active}"
    >
      ${label}
    </button>
  `;
}
