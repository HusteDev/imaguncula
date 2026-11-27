// SPDX-License-Identifier: MIT OR Apache-2.0
//! A fake desktop backend for component tests. It answers the Tauri commands
//! the frontend invokes through Tauri's own IPC mock, so components run
//! unchanged against deterministic data.

import { clearMocks, mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import { vi } from "vitest";

import type { OutputSpec } from "$lib/output";
import type { PresetSummary, ProjectState } from "$lib/project";
import { emptyProject } from "$lib/project";
import type { PreviewImage, SourceInfo } from "$lib/source";

interface TauriInternals {
  invoke: (command: string, args?: Record<string, unknown>, options?: unknown) => Promise<unknown>;
}

export type Handler = (args: Record<string, unknown>) => unknown;

export interface FakeDesktop {
  /** Every command invoked, in order, excluding the event plugin. */
  calls: Array<{ command: string; args: Record<string, unknown> }>;
  /** Results returned by successive open-dialog calls; `null` cancels. */
  openResults: Array<string | null>;
  /** Results returned by successive save-dialog calls; `null` cancels. */
  saveResults: Array<string | null>;
  /** Replaces the handler for one command. */
  handle(command: string, handler: Handler): void;
  /** Arguments of the most recent call to a command. */
  lastArgs(command: string): Record<string, unknown> | undefined;
  /** Events the frontend currently listens for. */
  listening: Set<string>;
  /** Delivers a backend event to the frontend's listeners. */
  emit(event: string, payload: unknown): Promise<void>;
}

const HASH = "c".repeat(64);

export const PRESETS: PresetSummary[] = [
  {
    id: "windows-desktop",
    version: 2,
    displayName: "Windows desktop",
    exclusiveGroup: "windows-icon",
    platform: "windows",
    sourceRoles: [{ role: "master", required: true, description: "Master artwork" }],
    outputIds: ["icon-16", "icon-32", "icon-256"],
    defaultOutputIds: ["icon-16", "icon-32"],
    optionalOutputGroups: [
      {
        id: "large",
        displayName: "Large icon",
        description: "Adds the 256 pixel icon.",
        defaultEnabled: false,
        outputs: ["icon-256"],
        packages: null,
      },
      {
        id: "packages-only",
        displayName: "Package group",
        description: "Groups packages, not outputs.",
        defaultEnabled: false,
        outputs: null,
        packages: ["ico"],
      },
    ],
  },
  {
    id: "windows-msix",
    version: 1,
    displayName: "Windows MSIX",
    exclusiveGroup: "windows-icon",
    platform: "windows",
    sourceRoles: [{ role: "master", required: true, description: "Master artwork" }],
    outputIds: ["msix-44"],
    defaultOutputIds: ["msix-44"],
    optionalOutputGroups: [],
  },
  {
    id: "android-adaptive",
    version: 1,
    displayName: "Android adaptive",
    exclusiveGroup: null,
    platform: "android",
    sourceRoles: [
      { role: "master", required: true, description: "Master artwork" },
      { role: "foreground", required: true, description: "Adaptive foreground layer." },
      { role: "monochrome", required: false, description: "Themed icon layer." },
    ],
    outputIds: ["adaptive-foreground"],
    defaultOutputIds: ["adaptive-foreground"],
    optionalOutputGroups: [],
  },
];

const PRESET_OUTPUTS: Record<
  string,
  { presetId: string; width: number; height: number; format: "png" | "svg"; role: string }
> = {
  "icon-16": { presetId: "windows-desktop", width: 16, height: 16, format: "png", role: "master" },
  "icon-32": { presetId: "windows-desktop", width: 32, height: 32, format: "png", role: "master" },
  "icon-256": {
    presetId: "windows-desktop",
    width: 256,
    height: 256,
    format: "png",
    role: "master",
  },
  "msix-44": { presetId: "windows-msix", width: 44, height: 44, format: "png", role: "master" },
  "adaptive-foreground": {
    presetId: "android-adaptive",
    width: 432,
    height: 432,
    format: "png",
    role: "foreground",
  },
};

export function sourceInfo(path: string, role: string, overrides: Partial<SourceInfo> = {}) {
  const info: SourceInfo = {
    role,
    path,
    format: path.endsWith(".png") ? "png" : "svg",
    byteLength: 4096,
    contentHash: HASH,
    dimensions: { width: 512, height: 512 },
    shape: "square",
    warnings: [],
    png: null,
    ...overrides,
  };
  return info;
}

function preview(args: Record<string, unknown>): PreviewImage {
  return {
    contentHash: HASH,
    width: Number(args.width),
    height: Number(args.height),
    pngBytes: [137, 80, 78, 71],
    cacheHit: false,
  };
}

/** Resolves output rows from the project the way the core does, simplified. */
export function resolveOutputs(project: ProjectState): OutputSpec[] {
  const sources = new Map(project.sources.map((source) => [source.role, source]));
  const selected = new Set(project.selectedPresets.map((preset) => preset.id));
  const rows: OutputSpec[] = [];

  const row = (
    id: string,
    spec: Omit<OutputSpec, "sourcePath" | "sourceContentHash" | "status" | "issues" | "id">,
  ) => {
    const override = project.outputOverrides[id];
    const source = override?.sourcePath
      ? { path: override.sourcePath, contentHash: override.sourceContentHash ?? HASH }
      : sources.get(spec.sourceRole);
    const width = override?.width ?? spec.width;
    const height = override?.height ?? spec.height;
    const upscale = source?.path.endsWith(".png") && (width ?? 0) > 128;
    const issues: OutputSpec["issues"] = [];
    if (!source) {
      issues.push({
        code: "source-missing",
        severity: "error",
        message: `The ${spec.sourceRole} source is missing.`,
      });
    } else if (upscale && !(override?.allowUpscale ?? false)) {
      issues.push({
        code: "raster-upscale-forbidden",
        severity: "error",
        message: "The raster source would be upscaled.",
      });
    }
    rows.push({
      ...spec,
      id,
      width,
      height,
      sourcePath: source?.path ?? null,
      sourceContentHash: source?.contentHash ?? null,
      sourceOverride: Boolean(override?.sourcePath),
      allowUpscale: override?.allowUpscale ?? false,
      status: issues.some((issue) => issue.severity === "error") ? "error" : "ready",
      issues,
    });
  };

  for (const id of project.enabledOutputs) {
    const spec = PRESET_OUTPUTS[id];
    if (!spec || !selected.has(spec.presetId)) continue;
    const preset = PRESETS.find((candidate) => candidate.id === spec.presetId);
    row(id, {
      presetId: spec.presetId,
      presetVersion: preset?.version ?? 1,
      included: true,
      platform: preset?.platform ?? "custom",
      usage: `${id} usage`,
      width: spec.width,
      height: spec.height,
      maximumBytes: null,
      format: spec.format,
      filename: `${project.baseFilename}-${id}.${spec.format}`,
      destination: preset?.platform ?? "custom",
      packagePath: `${preset?.platform}/${id}.${spec.format}`,
      packageId: "files",
      packageKind: "file",
      sourceRole: spec.role,
      sourceLabel: spec.role,
      sourceOverride: false,
      allowUpscale: false,
      flattenColor: null,
      safeZonePercent: id === "adaptive-foreground" ? 66 : null,
    });
  }
  for (const custom of project.customOutputs) {
    row(custom.id, {
      presetId: null,
      presetVersion: null,
      included: project.enabledOutputs.includes(custom.id),
      platform: "custom",
      usage: custom.usage,
      width: custom.format === "svg" ? null : custom.width,
      height: custom.format === "svg" ? null : custom.height,
      maximumBytes: null,
      format: custom.format,
      filename: custom.filename,
      destination: custom.destination,
      packagePath: `${custom.destination}/${custom.filename}`,
      packageId: "files",
      packageKind: "file",
      sourceRole: custom.sourceRole,
      sourceLabel: custom.sourceRole,
      sourceOverride: false,
      allowUpscale: false,
      flattenColor: null,
      safeZonePercent: null,
    });
  }
  return rows;
}

/**
 * Installs the fake backend. Call `uninstallDesktop` (or rely on the test
 * file's `afterEach`) to remove it.
 */
export function installDesktop(): FakeDesktop {
  const handlers = new Map<string, Handler>();
  const desktop: FakeDesktop = {
    calls: [],
    openResults: [],
    saveResults: [],
    listening: new Set(),
    handle(command, handler) {
      handlers.set(command, handler);
    },
    lastArgs(command) {
      return desktop.calls.filter((call) => call.command === command).at(-1)?.args;
    },
    async emit(event, payload) {
      await emit(event, payload);
    },
  };

  const project = (args: Record<string, unknown>) => args.project as ProjectState;
  handlers.set("application_name", () => "Imaguncula");
  handlers.set("new_project", (args) => ({
    ...emptyProject(),
    projectName: String(args.projectName),
    selectedPresets: [{ id: "windows-desktop", version: 2 }],
    enabledOutputs: ["icon-16", "icon-32"],
  }));
  handlers.set("list_presets", () => PRESETS);
  handlers.set("list_preset_upgrades", () => []);
  handlers.set("resolve_outputs", (args) => resolveOutputs(project(args)));
  handlers.set("import_source", (args) => sourceInfo(String(args.path), String(args.role)));
  handlers.set("render_preview", preview);
  handlers.set("save_project", () => null);
  handlers.set("plugin:dialog|open", () => desktop.openResults.shift() ?? null);
  handlers.set("plugin:dialog|save", () => desktop.saveResults.shift() ?? null);

  mockWindows("main");
  mockIPC(
    (command, args) => {
      const payload = (args ?? {}) as Record<string, unknown>;
      desktop.calls.push({ command, args: payload });
      const handler = handlers.get(command);
      if (!handler) return Promise.reject(new Error(`Unexpected command ${command}`));
      return handler(payload);
    },
    { shouldMockEvents: true },
  );
  // Record listener registration so tests can wait until the frontend is
  // subscribed before emitting events.
  const internals = (window as unknown as { __TAURI_INTERNALS__: TauriInternals })
    .__TAURI_INTERNALS__;
  const invoke = internals.invoke;
  internals.invoke = (command, args, options) => {
    if (command === "plugin:event|listen") desktop.listening.add(String(args?.event));
    return invoke(command, args, options);
  };
  // The IPC mock does not provide the hook that unlisten functions call.
  (window as unknown as Record<string, unknown>).__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: () => {},
  };
  (globalThis as { isTauri?: boolean }).isTauri = true;
  return desktop;
}

export function uninstallDesktop() {
  clearMocks();
  delete (window as unknown as Record<string, unknown>).__TAURI_EVENT_PLUGIN_INTERNALS__;
  delete (globalThis as { isTauri?: boolean }).isTauri;
}

/** jsdom has no object URLs; previews only need stable strings. */
export function stubObjectUrls() {
  let next = 0;
  const create = vi.fn(() => `blob:preview-${++next}`);
  const revoke = vi.fn();
  Object.assign(URL, { createObjectURL: create, revokeObjectURL: revoke });
  return { create, revoke };
}
