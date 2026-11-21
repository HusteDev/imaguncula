// SPDX-License-Identifier: MIT OR Apache-2.0
import { mkdir, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const destinationV1 = resolve(
  repositoryRoot,
  "crates/imaguncula-presets/presets/windows-msix/1/preset.json",
);
const destinationV2 = resolve(
  repositoryRoot,
  "crates/imaguncula-presets/presets/windows-msix/2/preset.json",
);

const outputs = [];
const files = [];
const groups = new Map();

function addOutput({ id, width, height, path, usage, group }) {
  outputs.push({
    id,
    sourceRole: "master",
    format: "png",
    width,
    height,
    usage,
    encoder: { type: "png", compression: "best", filter: "adaptive" },
  });
  files.push({ output: id, path });
  if (group) {
    const entries = groups.get(group) ?? [];
    entries.push(id);
    groups.set(group, entries);
  }
}

const scales = [
  [100, 1],
  [125, 1.25],
  [150, 1.5],
  [200, 2],
  [400, 4],
];

function scaledSize(value, multiplier) {
  return Math.round(value * multiplier);
}

function addScaleFamily({
  id,
  file,
  width,
  height = width,
  usage,
  group,
  idSuffix = "",
  fileQualifier = "",
}) {
  for (const [scale, multiplier] of scales) {
    addOutput({
      id: `msix-${id}-scale-${scale}${idSuffix}`,
      width: scaledSize(width, multiplier),
      height: scaledSize(height, multiplier),
      path: `Assets/${file}.scale-${scale}${fileQualifier}.png`,
      usage: `${usage} at ${scale}% scale.`,
      group,
    });
  }
}

const targetSizes = [16, 20, 24, 30, 32, 36, 40, 48, 60, 64, 72, 80, 96, 256];
for (const size of targetSizes) {
  addOutput({
    id: `msix-app-list-target-${size}`,
    width: size,
    height: size,
    path: `Assets/AppList.targetsize-${size}.png`,
    usage: `Required Windows app-list icon at the exact ${size}-pixel target size.`,
  });
  addOutput({
    id: `msix-app-list-target-${size}-dark-unplated`,
    width: size,
    height: size,
    path: `Assets/AppList.targetsize-${size}_altform-unplated.png`,
    usage: `Required unplated app-list icon for dark surfaces at ${size} pixels.`,
  });
  addOutput({
    id: `msix-app-list-target-${size}-light-unplated`,
    width: size,
    height: size,
    path: `Assets/AppList.targetsize-${size}_altform-lightunplated.png`,
    usage: `Required unplated app-list icon for light surfaces at ${size} pixels.`,
  });
}

addScaleFamily({
  id: "app-list",
  file: "AppList",
  width: 44,
  usage: "Square44x44Logo scale-qualified app-list asset",
});
addScaleFamily({
  id: "medium-tile",
  file: "MedTile",
  width: 150,
  usage: "Square150x150Logo medium tile and task-switcher asset",
});
addScaleFamily({
  id: "store-logo",
  file: "StoreLogo",
  width: 50,
  usage: "Required package logo for Microsoft Store publication",
});

addScaleFamily({
  id: "app-list-light",
  file: "AppList",
  width: 44,
  usage: "Optional Windows 10 light-theme app-list scale asset",
  group: "app-list-light-scale",
  idSuffix: "-altform-colorful-theme-light",
  fileQualifier: "_altform-colorful_theme-light",
});

for (const family of [
  ["small-tile", "SmallTile", 71, 71, "Square71x71Logo small tile"],
  ["wide-tile", "WideTile", 310, 150, "Wide310x150Logo wide tile"],
  ["large-tile", "LargeTile", 310, 310, "Square310x310Logo large tile"],
]) {
  addScaleFamily({
    id: family[0],
    file: family[1],
    width: family[2],
    height: family[3],
    usage: family[4],
    group: "windows-10-tiles",
  });
}

for (const family of [
  ["small-tile-light", "SmallTile", 71, 71, "Optional light-theme small tile"],
  ["medium-tile-light", "MedTile", 150, 150, "Optional light-theme medium tile"],
  ["wide-tile-light", "WideTile", 310, 150, "Optional light-theme wide tile"],
  ["large-tile-light", "LargeTile", 310, 310, "Optional light-theme large tile"],
]) {
  addScaleFamily({
    id: family[0],
    file: family[1],
    width: family[2],
    height: family[3],
    usage: family[4],
    group: "windows-10-light-tiles",
    idSuffix: "-altform-colorful-theme-light",
    fileQualifier: "_altform-colorful_theme-light",
  });
}

addScaleFamily({
  id: "splash",
  file: "SplashScreen",
  width: 620,
  height: 300,
  usage: "Optional splash screen",
  group: "splash-screen",
});
addScaleFamily({
  id: "splash-dark",
  file: "SplashScreen",
  width: 620,
  height: 300,
  usage: "Optional dark-theme splash screen",
  group: "splash-screen-themes",
  idSuffix: "-altform-colorful-theme-dark",
  fileQualifier: "_altform-colorful_theme-dark",
});
addScaleFamily({
  id: "splash-light",
  file: "SplashScreen",
  width: 620,
  height: 300,
  usage: "Optional light-theme splash screen",
  group: "splash-screen-themes",
  idSuffix: "-altform-colorful-theme-light",
  fileQualifier: "_altform-colorful_theme-light",
});
addScaleFamily({
  id: "badge",
  file: "BadgeLogo",
  width: 24,
  usage: "Optional monochrome Windows 10 lock-screen badge",
  group: "lock-screen-badge",
});
addScaleFamily({
  id: "store-logo-light",
  file: "StoreLogo",
  width: 50,
  usage: "Optional light-theme Microsoft Store package logo",
  group: "store-light-theme",
  idSuffix: "-altform-colorful-theme-light",
  fileQualifier: "_altform-colorful_theme-light",
});

const optionalGroupDescriptions = {
  "app-list-light-scale": [
    "Windows 10 app-list light theme",
    "Scale-qualified light-theme variants used by Windows 10.",
  ],
  "windows-10-tiles": [
    "Windows 10 tile families",
    "Small, wide, and large tiles. Windows 11 does not use them; the medium family remains enabled for Store compatibility.",
  ],
  "windows-10-light-tiles": [
    "Windows 10 light-theme tiles",
    "Light-theme variants for every Windows 10 tile family.",
  ],
  "splash-screen": ["Splash screen", "Scale-qualified default splash-screen artwork."],
  "splash-screen-themes": [
    "Splash-screen themes",
    "Dark- and light-theme variants of the splash screen.",
  ],
  "lock-screen-badge": [
    "Windows 10 lock-screen badge",
    "Monochrome badge assets for apps that register compatible background tasks.",
  ],
  "store-light-theme": [
    "Microsoft Store light theme",
    "Optional light-theme package-logo variants.",
  ],
};

const preset = {
  schemaVersion: 1,
  id: "windows-msix",
  version: 1,
  displayName: "Windows packaged app (MSIX/UWP)",
  description:
    "Manifest asset families for Windows packaged apps, including exact target sizes, unplated theme variants, scale resources, Store logos, and optional Windows 10 surfaces.",
  platform: "windows",
  lastReviewed: "2026-07-31",
  specSources: [
    "https://learn.microsoft.com/en-us/windows/apps/design/iconography/app-icon-construction",
    "https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-uap-visualelements",
    "https://learn.microsoft.com/en-us/windows/msix/package/create-app-package-with-makeappx-tool",
  ],
  sourceRoles: [
    {
      role: "master",
      required: true,
      description:
        "Square master artwork used for app-list, tile, package-logo, splash, and optional badge rows; replace individual outputs when theme-specific or non-square compositions are required.",
    },
  ],
  outputs,
  packages: [{ kind: "directory", id: "msix-assets", root: "msix", files }],
  validationRules: [
    { rule: "square-source", severity: "error", sourceRole: "master" },
    { rule: "no-upscale", severity: "error", sourceRole: "master" },
    {
      rule: "minimum-source-size",
      severity: "warning",
      sourceRole: "master",
      width: 1240,
      height: 1240,
    },
  ],
  optionalOutputGroups: [...groups.entries()].map(([id, groupOutputs]) => ({
    id,
    displayName: optionalGroupDescriptions[id][0],
    description: optionalGroupDescriptions[id][1],
    defaultEnabled: false,
    outputs: groupOutputs,
  })),
};

const presetV1 = structuredClone(preset);

for (const [scale, multiplier] of [
  [250, 2.5],
  [300, 3],
]) {
  for (const [id, file, width, height, usage] of [
    ["app-list", "AppList", 44, 44, "Intermediate-scale app-list asset"],
    ["small-tile", "SmallTile", 71, 71, "Intermediate-scale Windows 10 small tile"],
    ["medium-tile", "MedTile", 150, 150, "Intermediate-scale Windows 10 medium tile"],
    ["wide-tile", "WideTile", 310, 150, "Intermediate-scale Windows 10 wide tile"],
    ["large-tile", "LargeTile", 310, 310, "Intermediate-scale Windows 10 large tile"],
    ["store-logo", "StoreLogo", 50, 50, "Intermediate-scale package logo"],
  ]) {
    addOutput({
      id: `msix-${id}-scale-${scale}`,
      width: scaledSize(width, multiplier),
      height: scaledSize(height, multiplier),
      path: `Assets/${file}.scale-${scale}.png`,
      usage: `${usage} at ${scale}% scale.`,
      group: "intermediate-scales",
    });
  }
}
optionalGroupDescriptions["intermediate-scales"] = [
  "250% and 300% display scales",
  "Exact intermediate-scale assets for systems where avoiding nearest-neighbor scale selection matters.",
];

const presetV2 = {
  ...preset,
  version: 2,
  description:
    "Windows packaged-app manifest families with exact target sizes, unplated themes, required scale resources, and optional 250% and 300% assets.",
  outputs: structuredClone(outputs),
  packages: [{ kind: "directory", id: "msix-assets", root: "msix", files: structuredClone(files) }],
  optionalOutputGroups: [...groups.entries()].map(([id, groupOutputs]) => ({
    id,
    displayName: optionalGroupDescriptions[id][0],
    description: optionalGroupDescriptions[id][1],
    defaultEnabled: false,
    outputs: groupOutputs,
  })),
};

await mkdir(dirname(destinationV1), { recursive: true });
await mkdir(dirname(destinationV2), { recursive: true });
await Promise.all([
  writeFile(destinationV1, `${JSON.stringify(presetV1, null, 2)}\n`, "utf8"),
  writeFile(destinationV2, `${JSON.stringify(presetV2, null, 2)}\n`, "utf8"),
]);
