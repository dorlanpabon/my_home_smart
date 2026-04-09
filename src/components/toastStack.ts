import type { ToastMessage } from "../types/models";
import { escapeHtml } from "../utils/escape";

export function renderToastStack(toasts: ToastMessage[]): string {
  if (toasts.length === 0) {
    return "";
  }

  return `
    <div class="toast-stack" aria-live="polite">
      ${toasts
        .map(
          (toast) => `
            <div class="toast toast--${toast.tone}">
              <span>${escapeHtml(toast.message)}</span>
              <button class="icon-button" data-action="dismiss-toast" data-toast-id="${escapeHtml(toast.id)}">Dismiss</button>
            </div>
          `,
        )
        .join("")}
    </div>
  `;
}
