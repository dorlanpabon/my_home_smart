import type { Device } from "../types/models";

export function filterDevices(
  devices: Device[],
  searchQuery: string,
  statusFilter: "all" | "online" | "offline",
): Device[] {
  const normalizedQuery = searchQuery.trim().toLowerCase();

  return devices.filter((device) => {
    const matchesSearch =
      normalizedQuery.length === 0 ||
      device.name.toLowerCase().includes(normalizedQuery) ||
      device.id.toLowerCase().includes(normalizedQuery) ||
      device.channels.some((channel) =>
        channel.displayName.toLowerCase().includes(normalizedQuery),
      );

    const matchesFilter =
      statusFilter === "all" ||
      (statusFilter === "online" && device.online) ||
      (statusFilter === "offline" && !device.online);

    return matchesSearch && matchesFilter;
  });
}
