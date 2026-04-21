import type { Device } from "../types/models";

import { filterDevices } from "./deviceFilters";

function makeDevice(
  id: string,
  name: string,
  online: boolean,
  channelName: string,
): Device {
  return {
    id,
    name,
    online,
    inferredType: "Switch",
    gangCount: 1,
    channels: [
      {
        code: "switch_1",
        displayName: channelName,
        index: 1,
        currentState: false,
        controllable: true,
      },
    ],
    raw: {
      summary: {},
      details: {},
      functions: [],
      status: [],
      capabilities: [],
      specifications: {},
    },
  };
}

describe("filterDevices", () => {
  it("filters by online state and search query", () => {
    const devices = [
      makeDevice("alpha", "Living Room", true, "Ceiling"),
      makeDevice("beta", "Hallway", false, "Lamp"),
    ];

    const result = filterDevices(devices, "living", "online");

    expect(result).toHaveLength(1);
    expect(result[0]?.id).toBe("alpha");
  });

  it("matches channel names in the search query", () => {
    const devices = [
      makeDevice("alpha", "Living Room", true, "Desk lamp"),
      makeDevice("beta", "Kitchen", true, "Main light"),
    ];

    const result = filterDevices(devices, "desk", "all");

    expect(result).toHaveLength(1);
    expect(result[0]?.id).toBe("alpha");
  });

  it("supports the favorites filter", () => {
    const devices = [
      makeDevice("alpha", "Living Room", true, "Desk lamp"),
      makeDevice("beta", "Kitchen", true, "Main light"),
    ];

    const result = filterDevices(devices, "", "favorites", ["beta"]);

    expect(result).toHaveLength(1);
    expect(result[0]?.id).toBe("beta");
  });
});
