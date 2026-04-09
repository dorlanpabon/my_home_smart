import type { ActionLogEntry } from "../types/models";
import { escapeHtml } from "../utils/escape";
import { formatActionTimestamp } from "../utils/format";

export function renderActionLog(entries: ActionLogEntry[]): string {
  return `
    <aside class="log-panel">
      <div class="panel-heading">
        <p class="eyebrow">Recent actions</p>
        <h2>Local activity log</h2>
      </div>
      ${
        entries.length === 0
          ? `<div class="empty-state empty-state--inline"><strong>No recent actions</strong><p>Channel toggles will appear here after the first command.</p></div>`
          : `<ul class="log-list">
              ${entries
                .map(
                  (entry) => `
                    <li class="log-entry ${entry.success ? "is-success" : "is-error"}">
                      <div>
                        <strong>${escapeHtml(entry.deviceName ?? entry.deviceId ?? "Unknown device")}</strong>
                        <p>${escapeHtml(entry.message)}</p>
                      </div>
                      <span>${escapeHtml(formatActionTimestamp(entry))}</span>
                    </li>
                  `,
                )
                .join("")}
            </ul>`
      }
    </aside>
  `;
}
