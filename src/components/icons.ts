export function renderBulbIcon(active: boolean): string {
  return `
    <svg viewBox="0 0 24 24" aria-hidden="true" class="icon-svg">
      <path d="M9 18h6" />
      <path d="M10 21h4" />
      <path d="M8.8 14.5c-.9-.8-1.8-2-1.8-4a5 5 0 1 1 10 0c0 2-1 3.2-1.8 4" />
      <path d="M10 14.8h4" />
      <circle cx="12" cy="10.5" r="${active ? "1.2" : "0.7"}" />
    </svg>
  `;
}

export function renderPowerIcon(active: boolean): string {
  return `
    <svg viewBox="0 0 24 24" aria-hidden="true" class="icon-svg">
      <path d="M12 3v7" />
      <path d="M7.8 5.8A8 8 0 1 0 16.2 5.8" />
      ${active ? '<circle cx="12" cy="14" r="1.1" />' : ""}
    </svg>
  `;
}

export function renderSwitchIcon(): string {
  return `
    <svg viewBox="0 0 24 24" aria-hidden="true" class="icon-svg">
      <rect x="6" y="3.5" width="12" height="17" rx="2.8" />
      <path d="M12 7v10" />
      <circle cx="9" cy="18" r="0.7" />
      <circle cx="15" cy="18" r="0.7" />
    </svg>
  `;
}

export function renderRefreshIcon(): string {
  return `
    <svg viewBox="0 0 24 24" aria-hidden="true" class="icon-svg">
      <path d="M20 11a8 8 0 0 0-14.8-4" />
      <path d="M4 5v4h4" />
      <path d="M4 13a8 8 0 0 0 14.8 4" />
      <path d="M20 19v-4h-4" />
    </svg>
  `;
}

export function renderSettingsIcon(): string {
  return `
    <svg viewBox="0 0 24 24" aria-hidden="true" class="icon-svg">
      <circle cx="12" cy="12" r="2.6" />
      <path d="M19 12a7 7 0 0 0-.1-1l2-1.6-2-3.4-2.5 1a7.3 7.3 0 0 0-1.8-1l-.4-2.7H10l-.4 2.7a7.3 7.3 0 0 0-1.8 1l-2.5-1-2 3.4 2 1.6a7 7 0 0 0 0 2l-2 1.6 2 3.4 2.5-1a7.3 7.3 0 0 0 1.8 1l.4 2.7h4.2l.4-2.7a7.3 7.3 0 0 0 1.8-1l2.5 1 2-3.4-2-1.6c.1-.3.1-.7.1-1z" />
    </svg>
  `;
}

export function renderFavoriteIcon(active: boolean): string {
  return `
    <svg viewBox="0 0 24 24" aria-hidden="true" class="icon-svg">
      <path d="M12 3.8l2.5 5 5.5.8-4 3.9.9 5.5-4.9-2.6-4.9 2.6.9-5.5-4-3.9 5.5-.8z" ${active ? 'fill="currentColor"' : ""} />
    </svg>
  `;
}
