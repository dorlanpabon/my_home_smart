import type { AppState } from "../types/models";

import { renderActionLog } from "../components/actionLog";
import { renderConfigPanel } from "../components/configPanel";
import { renderDeviceGrid } from "../components/deviceGrid";
import { renderHeader } from "../components/header";
import { renderToastStack } from "../components/toastStack";

export function renderAppShell(
  state: AppState,
  visibleDevices: AppState["devices"],
): string {
  if (!state.hasConfig && !state.bootstrapping) {
    return `
      <main class="app-shell app-shell--setup">
        ${renderConfigPanel(state)}
        ${renderToastStack(state.toasts)}
      </main>
    `;
  }

  const developerMode = state.uiPreferences.viewMode === "developer";

  return `
    <main class="app-shell app-shell--${state.uiPreferences.viewMode}">
      <div class="app-shell__surface">
        ${renderHeader(state, visibleDevices.length)}
        <section class="workspace workspace--${state.uiPreferences.viewMode}">
          <div class="workspace__main">
            ${renderDeviceGrid(state, visibleDevices)}
          </div>
          ${
            developerMode
              ? `
                <div class="workspace__side">
                  ${renderActionLog(state.actionLog)}
                </div>
              `
              : ""
          }
        </section>
      </div>
      ${renderConfigPanel(state)}
      ${renderToastStack(state.toasts)}
    </main>
  `;
}
