import "./styles/app.css";

import { renderAppShell } from "./pages/appShell";
import { AppStore } from "./stores/appStore";

const root = document.querySelector<HTMLDivElement>("#app");

if (!root) {
  throw new Error("Unable to locate #app root element.");
}

const appRoot = root;

const store = new AppStore();
let queuedState = store.getState();
let renderScheduled = false;
let lastMarkup = "";

store.subscribe((state) => {
  queuedState = state;
  scheduleRender();
});

root.addEventListener("click", async (event) => {
  const target = event.target as HTMLElement | null;
  const actionTarget = target?.closest<HTMLElement>("[data-action]");
  if (!actionTarget) {
    return;
  }

  const action = actionTarget.dataset.action;
  switch (action) {
    case "open-config":
      store.openConfig();
      break;
    case "close-config":
      if (actionTarget.classList.contains("modal-backdrop") || actionTarget.dataset.action === "close-config") {
        store.closeConfig();
      }
      break;
    case "refresh-devices":
      await store.refreshDevices();
      break;
    case "test-connection":
      await store.testConnection();
      break;
    case "set-filter":
      if (actionTarget.dataset.filter === "all" || actionTarget.dataset.filter === "online" || actionTarget.dataset.filter === "offline") {
        store.setStatusFilter(actionTarget.dataset.filter);
      }
      break;
    case "set-view-mode":
      if (actionTarget.dataset.viewMode === "user" || actionTarget.dataset.viewMode === "developer") {
        await store.setViewMode(actionTarget.dataset.viewMode);
      }
      break;
    case "toggle-channel":
      if (actionTarget.dataset.deviceId && actionTarget.dataset.channelCode && actionTarget.dataset.value) {
        await store.toggleChannel(
          actionTarget.dataset.deviceId,
          actionTarget.dataset.channelCode,
          actionTarget.dataset.value === "true",
        );
      }
      break;
    case "copy-device-id":
      if (actionTarget.dataset.deviceId) {
        await copyText(actionTarget.dataset.deviceId);
      }
      break;
    case "dismiss-toast":
      if (actionTarget.dataset.toastId) {
        store.removeToast(actionTarget.dataset.toastId);
      }
      break;
    default:
      break;
  }
});

root.addEventListener("input", (event) => {
  const target = event.target as HTMLInputElement | null;
  if (!target || !target.name) {
    return;
  }

  if (
    target.name === "clientId" ||
    target.name === "clientSecret" ||
    target.name === "baseUrl" ||
    target.name === "regionLabel"
  ) {
    store.updateConfigDraft(target.name, target.value);
    return;
  }

  if (target.name === "searchQuery") {
    store.setSearchQuery(target.value);
  }
});

root.addEventListener("submit", async (event) => {
  const form = event.target as HTMLFormElement | null;
  if (!form) {
    return;
  }

  event.preventDefault();
  const data = new FormData(form);

  switch (form.dataset.form) {
    case "config":
      await store.saveConfig();
      break;
    case "device-alias":
      await store.saveDeviceAlias({
        deviceId: String(data.get("deviceId") ?? ""),
        alias: String(data.get("alias") ?? ""),
      });
      break;
    case "channel-alias":
      await store.saveChannelAlias({
        deviceId: String(data.get("deviceId") ?? ""),
        channelCode: String(data.get("channelCode") ?? ""),
        alias: String(data.get("alias") ?? ""),
      });
      break;
    default:
      break;
  }
});

void store.bootstrap();

window.addEventListener("focus", () => {
  void store.refreshStatuses();
});

document.addEventListener("visibilitychange", () => {
  if (document.visibilityState === "visible") {
    void store.refreshStatuses();
  }
});

function scheduleRender(): void {
  if (renderScheduled) {
    return;
  }

  renderScheduled = true;
  window.requestAnimationFrame(() => {
    renderScheduled = false;
    const nextMarkup = renderAppShell(queuedState, store.getVisibleDevices());
    if (nextMarkup === lastMarkup) {
      return;
    }

    appRoot.innerHTML = nextMarkup;
    lastMarkup = nextMarkup;
  });
}

async function copyText(value: string): Promise<void> {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(value);
    return;
  }

  const element = document.createElement("textarea");
  element.value = value;
  document.body.append(element);
  element.select();
  document.execCommand("copy");
  element.remove();
}
