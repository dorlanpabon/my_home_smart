import type { Device } from "../types/models";

export function filterDevices(
  devices: Device[],
  searchQuery: string,
  statusFilter: "all" | "online" | "offline" | "favorites",
  favoriteDeviceIds: string[] = [],
): Device[] {
  const normalizedQuery = searchQuery.trim().toLowerCase();
  const favoriteSet = new Set(favoriteDeviceIds);

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
      (statusFilter === "offline" && !device.online) ||
      (statusFilter === "favorites" && favoriteSet.has(device.id));

    return matchesSearch && matchesFilter;
  });
}
