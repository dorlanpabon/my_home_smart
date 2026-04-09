import type { AppState } from "../types/models";
import { escapeHtml } from "../utils/escape";

export function renderConfigPanel(state: AppState): string {
  const inner = `
    <section class="config-card">
      <div class="config-card__intro">
        <p class="eyebrow">Cloud setup</p>
        <h2>${state.hasConfig ? "Update Tuya connection" : "Connect your Tuya Cloud project"}</h2>
        <p class="subtle">
          Credentials stay in the Rust backend and are stored in the desktop app data directory.
        </p>
        ${
          state.environmentMessage
            ? `<p class="banner banner--warning">${escapeHtml(state.environmentMessage)}</p>`
            : ""
        }
      </div>

      <form class="config-form" data-form="config">
        <label>
          <span>Client ID</span>
          <input type="text" name="clientId" value="${escapeHtml(state.configDraft.clientId)}" autocomplete="off" />
        </label>
        <label>
          <span>Client Secret</span>
          <input type="password" name="clientSecret" value="${escapeHtml(state.configDraft.clientSecret)}" autocomplete="off" placeholder="${state.config?.clientSecretPresent ? "Leave blank to keep saved secret" : "Enter secret"}" />
        </label>
        <label>
          <span>Base URL</span>
          <input type="url" name="baseUrl" value="${escapeHtml(state.configDraft.baseUrl)}" autocomplete="off" />
        </label>
        <label>
          <span>Region label</span>
          <input type="text" name="regionLabel" value="${escapeHtml(state.configDraft.regionLabel)}" autocomplete="off" />
        </label>
        <div class="config-form__status">
          <span>Default region: Western America Data Center</span>
          ${
            state.config?.clientSecretPresent
              ? `<span>Saved secret ${escapeHtml(state.config.clientSecretMasked)}</span>`
              : ""
          }
        </div>
        <div class="config-form__actions">
          <button
            type="button"
            class="button button--secondary"
            data-action="test-connection"
            ${state.testingConnection ? "disabled" : ""}
          >
            ${state.testingConnection ? "Testing..." : "Test connection"}
          </button>
          ${
            state.hasConfig
              ? `<button type="button" class="button button--ghost" data-action="close-config">Close</button>`
              : ""
          }
          <button
            type="submit"
            class="button button--primary"
            ${state.savingConfig ? "disabled" : ""}
          >
            ${state.savingConfig ? "Saving..." : "Save configuration"}
          </button>
        </div>
      </form>
    </section>
  `;

  if (!state.hasConfig) {
    return `<section class="onboarding-shell">${inner}</section>`;
  }

  if (!state.configOpen) {
    return "";
  }

  return `
    <div class="modal-backdrop" data-action="close-config">
      <div class="modal" role="dialog" aria-modal="true" aria-label="Tuya configuration">
        ${inner}
      </div>
    </div>
  `;
}
