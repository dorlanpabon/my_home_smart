import type { ActionLogEntry, Device } from "../types/models";

export function formatConnectionState(state: string): string {
  switch (state) {
    case "connected":
      return "Connected";
    case "needs_config":
      return "Setup required";
    case "error":
      return "Connection issue";
    default:
      return "Unknown";
  }
}

export function formatDeviceSubtitle(device: Device): string {
  const parts = [device.inferredType];
  if (device.category) {
    parts.push(`Category ${device.category}`);
  }
  return parts.join(" - ");
}

export function formatLastSeen(timestampMs?: number | null): string {
  if (!timestampMs) {
    return "Not checked yet";
  }

  return new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    month: "short",
    day: "numeric",
  }).format(timestampMs);
}

export function formatActionTimestamp(entry: ActionLogEntry): string {
  return new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(entry.timestampMs);
}
