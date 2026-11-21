// SPDX-License-Identifier: MIT OR Apache-2.0
export const PROJECT_SCHEMA_VERSION = 2;

export interface ProjectSource {
  role: string;
  path: string;
  contentHash: string;
}

export interface SelectedPreset {
  id: string;
  version: number;
}

export interface PresetSummary {
  id: string;
  version: number;
  displayName: string;
  exclusiveGroup: string | null;
  platform: "windows" | "web" | "linux" | "macos" | "ios" | "ipados" | "android" | "custom";
  sourceRoles: PresetSourceRole[];
  outputIds: string[];
  defaultOutputIds: string[];
  optionalOutputGroups: PresetOptionalOutputGroup[];
}

export interface PresetSourceRole {
  role: string;
  required: boolean;
  description: string;
}

export interface PresetOptionalOutputGroup {
  id: string;
  displayName: string;
  description: string;
  defaultEnabled: boolean;
  outputs: string[] | null;
  packages: string[] | null;
}

export interface CustomOutput {
  id: string;
  width: number;
  height: number;
  format: "png" | "svg";
  filename: string;
  destination: string;
  sourceRole: string;
  usage: string;
}

export interface OutputOverride {
  width: number | null;
  height: number | null;
  sourceRole: string | null;
  sourcePath: string | null;
  sourceContentHash: string | null;
  allowUpscale: boolean;
}

export function emptyOutputOverride(): OutputOverride {
  return {
    width: null,
    height: null,
    sourceRole: null,
    sourcePath: null,
    sourceContentHash: null,
    allowUpscale: false,
  };
}

export interface ExportSettings {
  createCombinedZip: boolean;
  overwriteConfirmed: boolean;
}

export interface AcceptedWarning {
  code: string;
  outputId: string | null;
}

export interface PresetMigrationAcceptance {
  presetId: string;
  fromVersion: number;
  toVersion: number;
  diffFingerprint: string;
}

export interface PresetOutputRename {
  from: string;
  to: string;
}

export interface PresetDiff {
  presetId: string;
  fromVersion: number;
  toVersion: number;
  addedOutputs: string[];
  removedOutputs: string[];
  renamedOutputs: PresetOutputRename[];
}

export interface PresetUpgrade {
  diff: PresetDiff;
  diffFingerprint: string;
}

export interface ProjectState {
  schemaVersion: number;
  projectName: string;
  baseFilename: string;
  sources: ProjectSource[];
  selectedPresets: SelectedPreset[];
  enabledOutputs: string[];
  customOutputs: CustomOutput[];
  outputOverrides: Record<string, OutputOverride>;
  exportSettings: ExportSettings;
  acceptedWarnings: AcceptedWarning[];
  presetMigrations: PresetMigrationAcceptance[];
}

export interface ExternalSourceReference {
  reference: string;
  path: string;
}

export interface ProjectLoad {
  project: ProjectState;
  migrationNotices: string[];
  externalSources: ExternalSourceReference[];
}

export interface ProjectValidationIssue {
  code: string;
  path: string;
  message: string;
}

export interface ProjectFileError {
  kind: string;
  path?: string;
  message?: string;
  issues?: ProjectValidationIssue[];
  maximum_bytes?: number;
  actual_bytes?: number;
  found?: number | null;
  supported?: number;
}

export function emptyProject(): ProjectState {
  return {
    schemaVersion: PROJECT_SCHEMA_VERSION,
    projectName: "Untitled project",
    baseFilename: "untitled-project",
    sources: [],
    selectedPresets: [
      { id: "windows-desktop", version: 2 },
      { id: "web-pwa", version: 1 },
    ],
    enabledOutputs: [
      "icon-16",
      "icon-24",
      "icon-20",
      "icon-32",
      "icon-30",
      "icon-36",
      "icon-40",
      "icon-42",
      "icon-48",
      "icon-256",
      "icon-60",
      "icon-64",
      "icon-72",
      "icon-80",
      "icon-96",
      "favicon-16",
      "favicon-32",
      "favicon-48",
      "favicon-svg",
      "apple-touch-180",
      "pwa-192",
      "pwa-512",
      "pwa-maskable-512",
    ],
    customOutputs: [],
    outputOverrides: {},
    exportSettings: {
      createCombinedZip: true,
      overwriteConfirmed: false,
    },
    acceptedWarnings: [],
    presetMigrations: [],
  };
}

export function projectErrorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (!error || typeof error !== "object") {
    return "The project operation failed.";
  }

  const projectError = error as ProjectFileError;
  if (projectError.issues?.length) {
    return projectError.issues.map((issue) => `${issue.path}: ${issue.message}`).join("; ");
  }
  if (projectError.message) {
    return projectError.message;
  }

  switch (projectError.kind) {
    case "invalid-file-name":
      return "Project files must use the .imaguncula.json extension.";
    case "file-too-large":
      return "The selected project file exceeds the 1 MiB safety limit.";
    case "unsupported-schema-version":
      return `This project uses unsupported schema version ${projectError.found ?? "unknown"}.`;
    case "validation":
      return "The project contains invalid values.";
    default:
      return "The project operation failed.";
  }
}
