// SPDX-License-Identifier: MIT OR Apache-2.0
import type { CustomOutput, PresetSummary } from "./project";

export type OutputStatus = "ready" | "warning" | "error";
export type OutputIssueSeverity = "warning" | "error";

export interface OutputIssue {
  code: string;
  severity: OutputIssueSeverity;
  message: string;
}

export interface OutputSpec {
  id: string;
  presetId: string | null;
  presetVersion: number | null;
  included: boolean;
  platform: string;
  usage: string;
  width: number | null;
  height: number | null;
  maximumBytes: number | null;
  format: "png" | "svg";
  filename: string;
  destination: string;
  packagePath: string;
  packageId: string;
  packageKind: "file" | "container-entry" | "directory-file";
  sourceRole: string;
  sourceLabel: string;
  sourcePath: string | null;
  sourceContentHash: string | null;
  sourceOverride: boolean;
  allowUpscale: boolean;
  flattenColor: { red: number; green: number; blue: number } | null;
  safeZonePercent: number | null;
  status: OutputStatus;
  issues: OutputIssue[];
}

export interface OutputPreviewLayout {
  renderWidth: number;
  renderHeight: number;
  displayWidth: number;
  displayHeight: number;
  integerZoom: number | null;
}

export interface DestinationInspection {
  path: string;
  state:
    | "missing"
    | "empty-directory"
    | "non-empty-package-directory"
    | "non-empty-directory"
    | "non-directory";
  requiresConfirmation: boolean;
}

export interface ExportProgressEvent {
  jobId: string;
  completed: number;
  total: number;
  currentOutput: string | null;
}

export interface GeneratedFile {
  outputId: string;
  path: string;
  width: number | null;
  height: number | null;
  format: string;
  sourceLabel: string;
  sourcePath: string;
  sourceContentHash: string;
}

export interface AdditionalSourceRoleRequirement {
  role: string;
  outputIds: string[];
  configured: boolean;
}

export interface AdditionalSourceRoleOutputType {
  presetId: string;
  displayName: string;
  required: boolean;
}

export interface AdditionalSourceRoleDependency {
  role: string;
  required: boolean;
  description: string;
  outputTypes: AdditionalSourceRoleOutputType[];
}

export interface ReportWarning {
  outputId: string;
  code: string;
  message: string;
  accepted: boolean;
}

export interface GenerationReport {
  projectName: string;
  baseFilename: string;
  files: GeneratedFile[];
  warnings: ReportWarning[];
}

export interface ExportReport {
  destination: string;
  report: GenerationReport;
  reportMarkdown: string;
}

export interface ExportFinishedEvent {
  jobId: string;
  report: ExportReport | null;
  error: unknown | null;
}

export type CustomOutputField = "width" | "height" | "filename" | "destination" | "usage";

export function outputPreviewLayout(
  width: number | null,
  height: number | null,
  maximumRenderDimension = 256,
  maximumDisplayDimension = 64,
): OutputPreviewLayout {
  const naturalWidth = validPreviewDimension(width, 128);
  const naturalHeight = validPreviewDimension(height, 128);
  const renderLimit = validPreviewDimension(maximumRenderDimension, 256);
  const displayLimit = validPreviewDimension(maximumDisplayDimension, 64);
  const renderScale = Math.min(1, renderLimit / Math.max(naturalWidth, naturalHeight));
  const renderWidth = Math.max(1, Math.round(naturalWidth * renderScale));
  const renderHeight = Math.max(1, Math.round(naturalHeight * renderScale));
  const largestRenderedDimension = Math.max(renderWidth, renderHeight);

  if (largestRenderedDimension <= displayLimit) {
    const integerZoom = Math.max(1, Math.floor(displayLimit / largestRenderedDimension));
    return {
      renderWidth,
      renderHeight,
      displayWidth: renderWidth * integerZoom,
      displayHeight: renderHeight * integerZoom,
      integerZoom,
    };
  }

  const displayScale = displayLimit / largestRenderedDimension;
  return {
    renderWidth,
    renderHeight,
    displayWidth: Math.max(1, Math.round(renderWidth * displayScale)),
    displayHeight: Math.max(1, Math.round(renderHeight * displayScale)),
    integerZoom: null,
  };
}

function validPreviewDimension(value: number | null, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) && value > 0
    ? Math.round(value)
    : fallback;
}

export function additionalSourceRoleRequirements(
  outputs: readonly OutputSpec[],
): AdditionalSourceRoleRequirement[] {
  const requirements = new Map<string, AdditionalSourceRoleRequirement>();

  for (const output of outputs) {
    if (!output.included || output.sourceRole === "master" || output.sourceRole === "override") {
      continue;
    }

    const requirement = requirements.get(output.sourceRole) ?? {
      role: output.sourceRole,
      outputIds: [],
      configured: true,
    };
    requirement.outputIds.push(output.id);
    requirement.configured &&= Boolean(output.sourcePath && output.sourceContentHash);
    requirements.set(output.sourceRole, requirement);
  }

  return [...requirements.values()].sort((left, right) => left.role.localeCompare(right.role));
}

export function additionalSourceRoleNotice(preset: PresetSummary): string | null {
  const additionalRoles = preset.sourceRoles.filter((role) => role.role !== "master");
  if (!additionalRoles.length) return null;

  const required = additionalRoles.filter((role) => role.required).map((role) => role.role);
  const optional = additionalRoles.filter((role) => !role.required).map((role) => role.role);
  const messages: string[] = [];
  if (required.length) {
    messages.push(`Requires additional sources: ${formatList(required)}`);
  }
  if (optional.length) {
    messages.push(`Optional source${optional.length === 1 ? "" : "s"}: ${formatList(optional)}`);
  }
  return `${messages.join(". ")}.`;
}

export function additionalSourceRoleDependencies(
  presets: readonly PresetSummary[],
  selectedPresetIds: readonly string[],
): AdditionalSourceRoleDependency[] {
  const selected = new Set(selectedPresetIds);
  const dependencies = new Map<string, AdditionalSourceRoleDependency>();

  for (const preset of presets) {
    if (!selected.has(preset.id)) continue;
    for (const sourceRole of preset.sourceRoles) {
      if (sourceRole.role === "master") continue;
      const dependency = dependencies.get(sourceRole.role) ?? {
        role: sourceRole.role,
        required: false,
        description: sourceRole.description,
        outputTypes: [],
      };
      dependency.required ||= sourceRole.required;
      dependency.outputTypes.push({
        presetId: preset.id,
        displayName: preset.displayName,
        required: sourceRole.required,
      });
      dependencies.set(sourceRole.role, dependency);
    }
  }

  return [...dependencies.values()]
    .map((dependency) => ({
      ...dependency,
      outputTypes: dependency.outputTypes.sort((left, right) =>
        left.displayName.localeCompare(right.displayName),
      ),
    }))
    .sort((left, right) => left.role.localeCompare(right.role));
}

function formatList(values: readonly string[]): string {
  if (values.length < 2) return values[0] ?? "";
  if (values.length === 2) return `${values[0]} and ${values[1]}`;
  return `${values.slice(0, -1).join(", ")}, and ${values.at(-1)}`;
}

export function reportDimensions(file: GeneratedFile): string {
  if (file.width !== null && file.height !== null) return `${file.width}×${file.height}`;

  switch (file.format.toLowerCase()) {
    case "ico":
    case "icns":
      return "Multi-size";
    case "svg":
      return "Scalable";
    default:
      return "N/A";
  }
}

export function nextCustomOutput(existing: CustomOutput[], baseFilename: string): CustomOutput {
  const used = new Set(existing.map((output) => output.id));
  let sequence = existing.length + 1;
  let id = `custom-${sequence}`;
  while (used.has(id)) {
    sequence += 1;
    id = `custom-${sequence}`;
  }
  const safeBase = baseFilename || "icon";
  return {
    id,
    width: 64,
    height: 64,
    format: "png",
    filename: `${safeBase}-${sequence}.png`,
    destination: "custom",
    sourceRole: "master",
    usage: "Custom application icon",
  };
}

export function exportErrorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (!error || typeof error !== "object") return "The export failed.";
  const value = error as Record<string, unknown>;
  if (value.error) return exportErrorMessage(value.error);
  if (Array.isArray(value.issues) && value.issues.length) {
    return value.issues
      .map((issue) => {
        if (!issue || typeof issue !== "object") return "Invalid project value";
        const typed = issue as Record<string, unknown>;
        return `${String(typed.path ?? "project")}: ${String(typed.message ?? "invalid value")}`;
      })
      .join("; ");
  }
  switch (value.kind) {
    case "no-included-outputs":
      return "Enable at least one custom output before creating a package.";
    case "outputs-not-ready":
      return "Every included output must be ready before export.";
    case "cancelled":
      return "Export cancelled. No incomplete package was committed.";
    case "destination-exists":
      return "The export destination exists and requires explicit overwrite confirmation.";
    case "destination-not-directory":
      return "The export destination exists but is not a directory.";
    case "invalid-destination":
      return "The selected export destination is not usable.";
    case "source-changed":
      return "A source changed after output review. Refresh the table before exporting.";
    case "metadata-reference-missing":
      return "A generated metadata file references a disabled output. Re-enable that output before exporting.";
    case "metadata-condition-incomplete":
      return "A generated metadata package is only partly enabled. Enable or disable its complete output group.";
    case "metadata":
      return "Generated metadata could not be validated.";
    case "unsupported-container":
    case "container":
      return "A selected preset container could not be encoded.";
    case "zip":
      return "The combined ZIP could not be created or verified.";
    case "job-already-running":
      return "Another export is already running.";
    case "jobs-unavailable":
      return "The export job manager is unavailable. Restart the application.";
    case "verification":
      return "The staged package did not match the reviewed output list.";
    case "write":
    case "commit":
      return "The package could not be written or committed.";
    default:
      return typeof value.message === "string" ? value.message : "The export failed.";
  }
}
