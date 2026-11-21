<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount, tick } from "svelte";

  import OutputTable from "$lib/components/OutputTable.svelte";
  import {
    additionalSourceRoleDependencies,
    additionalSourceRoleNotice,
    exportErrorMessage,
    nextCustomOutput,
    reportDimensions,
    type AdditionalSourceRoleDependency,
    type CustomOutputField,
    type DestinationInspection,
    type ExportFinishedEvent,
    type ExportProgressEvent,
    type ExportReport,
    type OutputSpec,
  } from "$lib/output";
  import {
    emptyOutputOverride,
    emptyProject,
    projectErrorMessage,
    type ProjectLoad,
    type ProjectState,
    type PresetOptionalOutputGroup,
    type PresetSummary,
    type PresetUpgrade,
  } from "$lib/project";
  import {
    sourceOperationErrorMessage,
    sourceWarningMessage,
    type PreviewImage,
    type SourceInfo,
  } from "$lib/source";

  let project = $state<ProjectState>(emptyProject());
  let currentPath = $state<string | null>(null);
  let shellStatus = $state("Loading desktop core");
  let desktopRuntime = $state(false);
  let operationStatus = $state("Unsaved project");
  let operationError = $state("");
  let dirty = $state(false);
  let busy = $state(false);
  let sourceInfo = $state<SourceInfo | null>(null);
  let roleSourceInfo = $state<Record<string, SourceInfo>>({});
  let roleSourceErrors = $state<Record<string, string>>({});
  let sourceError = $state("");
  let dragActive = $state(false);
  let preview = $state<PreviewImage | null>(null);
  let previewUrl = $state("");
  let previewWidth = $state(64);
  let previewHeight = $state(64);
  let previewZoom = $state(1);
  let previewBackground = $state<"checkerboard" | "dark" | "light" | "transparent">("checkerboard");
  let displayPixelRatio = $state(1);
  let outputs = $state<OutputSpec[]>([]);
  let outputTable = $state<OutputTable | null>(null);
  let availablePresets = $state<PresetSummary[]>([]);
  let presetUpgrades = $state<PresetUpgrade[]>([]);
  let outputError = $state("");
  let outputBusy = $state(false);
  let outputResolutionVersion = 0;
  let activeExportJob = $state<string | null>(null);
  let exportProgress = $state<ExportProgressEvent | null>(null);
  let exportError = $state("");
  let exportReport = $state<ExportReport | null>(null);
  let pendingExportStart = false;

  onMount(() => {
    let disposed = false;
    let stopDragDrop: (() => void) | undefined;
    const stopDesktopEvents: Array<() => void> = [];
    displayPixelRatio = Math.max(1, window.devicePixelRatio || 1);
    desktopRuntime = isTauri();

    async function connectDesktop() {
      try {
        const [name, initialProject, presets] = await Promise.all([
          invoke<string>("application_name"),
          invoke<ProjectState>("new_project", { projectName: "Untitled project" }),
          invoke<PresetSummary[]>("list_presets"),
        ]);
        if (disposed) return;
        project = initialProject;
        availablePresets = presets;
        shellStatus = `${name} desktop core connected`;
        await refreshOutputs();
        await refreshPresetUpgrades();
      } catch {
        shellStatus = "Frontend preview mode";
      }

      try {
        const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
          if (event.payload.type === "over") {
            dragActive = true;
          } else if (event.payload.type === "leave") {
            dragActive = false;
          } else if (event.payload.type === "drop") {
            dragActive = false;
            const [path] = event.payload.paths;
            if (path) void importMasterSource(path, true);
          }
        });
        if (disposed) unlisten();
        else stopDragDrop = unlisten;
      } catch {
        dragActive = false;
      }

      try {
        const [stopProgress, stopFinished] = await Promise.all([
          listen<ExportProgressEvent>("export-progress", (event) => {
            if (pendingExportStart && !activeExportJob) activeExportJob = event.payload.jobId;
            if (event.payload.jobId === activeExportJob) exportProgress = event.payload;
          }),
          listen<ExportFinishedEvent>("export-finished", (event) => {
            if (event.payload.jobId !== activeExportJob && !pendingExportStart) return;
            pendingExportStart = false;
            activeExportJob = null;
            exportProgress = null;
            if (event.payload.report) {
              exportReport = event.payload.report;
              exportError = "";
              operationStatus = `Created ${event.payload.report.report.files.length} output file(s)`;
            } else {
              exportReport = null;
              exportError = exportErrorMessage(event.payload.error);
              operationStatus = "Export failed";
            }
          }),
        ]);
        if (disposed) {
          stopProgress();
          stopFinished();
        } else {
          stopDesktopEvents.push(stopProgress, stopFinished);
        }
      } catch {
        // Frontend preview mode has no desktop event bridge.
      }
    }

    void connectDesktop();
    return () => {
      disposed = true;
      stopDragDrop?.();
      for (const stop of stopDesktopEvents) stop();
      releasePreviewUrl();
    };
  });

  function markDirty() {
    dirty = true;
    operationStatus = "Unsaved changes";
    operationError = "";
  }

  async function refreshOutputs() {
    const version = ++outputResolutionVersion;
    outputBusy = true;
    try {
      const resolved = await invoke<OutputSpec[]>("resolve_outputs", { project });
      if (version !== outputResolutionVersion) return;
      outputs = resolved;
      outputError = "";
    } catch (error) {
      if (version !== outputResolutionVersion) return;
      outputError = exportErrorMessage(error);
    } finally {
      if (version === outputResolutionVersion) outputBusy = false;
    }
  }

  async function refreshPresetUpgrades() {
    try {
      presetUpgrades = await invoke<PresetUpgrade[]>("list_preset_upgrades", { project });
    } catch {
      presetUpgrades = [];
    }
  }

  async function acceptPresetUpgrade(upgrade: PresetUpgrade) {
    busy = true;
    operationError = "";
    try {
      project = await invoke<ProjectState>("apply_preset_upgrade", {
        project,
        presetId: upgrade.diff.presetId,
        targetVersion: upgrade.diff.toVersion,
        acceptedDiffFingerprint: upgrade.diffFingerprint,
      });
      markDirty();
      operationStatus = `${upgrade.diff.presetId} migrated to version ${upgrade.diff.toVersion}`;
      await Promise.all([refreshOutputs(), refreshPresetUpgrades()]);
    } catch (error) {
      operationError = projectErrorMessage(error);
    } finally {
      busy = false;
    }
  }

  function canDiscardChanges(): boolean {
    return !dirty || window.confirm("Discard the unsaved changes to this project?");
  }

  function releasePreviewUrl() {
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    previewUrl = "";
  }

  function clearSourceUi() {
    sourceInfo = null;
    roleSourceInfo = {};
    roleSourceErrors = {};
    sourceError = "";
    preview = null;
    releasePreviewUrl();
  }

  function sourceFilename(path: string): string {
    const parts = path.split(/[\\/]/);
    return parts[parts.length - 1] || path;
  }

  function setInitialPreviewSize(info: SourceInfo) {
    previewWidth = Math.max(1, Math.min(64, Math.floor(info.dimensions.width)));
    previewHeight = Math.max(1, Math.min(64, Math.floor(info.dimensions.height)));
  }

  async function renderSourcePreview(info: SourceInfo | null = sourceInfo) {
    if (!info) return;

    sourceError = "";
    try {
      const rendered = await invoke<PreviewImage>("render_preview", {
        path: info.path,
        role: info.role,
        width: Math.max(1, Math.round(previewWidth)),
        height: Math.max(1, Math.round(previewHeight)),
        allowUpscale: false,
      });
      releasePreviewUrl();
      const bytes = Uint8Array.from(rendered.pngBytes);
      previewUrl = URL.createObjectURL(new Blob([bytes.buffer], { type: "image/png" }));
      preview = rendered;
    } catch (error) {
      preview = null;
      releasePreviewUrl();
      sourceError = sourceOperationErrorMessage(error);
    }
  }

  async function importMasterSource(path: string, persist: boolean, expectedHash?: string) {
    busy = true;
    sourceError = "";
    try {
      const info = await invoke<SourceInfo>("import_source", { path, role: "master" });
      sourceInfo = info;
      setInitialPreviewSize(info);
      const sourceChanged = Boolean(expectedHash && expectedHash !== info.contentHash);
      if (persist) {
        project.sources = [
          ...project.sources.filter((source) => source.role !== "master"),
          { role: "master", path: info.path, contentHash: info.contentHash },
        ];
        markDirty();
        operationStatus = "Master source replaced";
      }
      await renderSourcePreview(info);
      await refreshOutputs();
      if (sourceChanged) sourceError = "The source contents changed since this project was saved.";
    } catch (error) {
      sourceError = sourceOperationErrorMessage(error);
    } finally {
      busy = false;
    }
  }

  async function chooseMasterSource() {
    sourceError = "";
    if (!desktopRuntime) {
      sourceError =
        "Source selection requires the Tauri desktop runtime. Run pnpm tauri dev; installation is not required.";
      return;
    }
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Icon source", extensions: ["svg", "png"] }],
      });
      if (typeof path === "string") await importMasterSource(path, true);
    } catch (error) {
      sourceError = sourceOperationErrorMessage(error);
    }
  }

  async function inspectRoleSource(
    role: string,
    path: string,
    persist: boolean,
    expectedHash?: string,
  ) {
    const info = await invoke<SourceInfo>("import_source", { path, role });
    roleSourceInfo = { ...roleSourceInfo, [role]: info };
    const sourceChanged = Boolean(expectedHash && expectedHash !== info.contentHash);
    roleSourceErrors = {
      ...roleSourceErrors,
      [role]: sourceChanged ? "The source contents changed since this project was saved." : "",
    };
    if (persist) {
      project.sources = [
        ...project.sources.filter((source) => source.role !== role),
        { role, path: info.path, contentHash: info.contentHash },
      ];
      markDirty();
      operationStatus = `${role} source replaced`;
      await refreshOutputs();
    }
  }

  async function chooseRoleSource(role: string) {
    busy = true;
    roleSourceErrors = { ...roleSourceErrors, [role]: "" };
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Icon source", extensions: ["svg", "png"] }],
      });
      if (typeof path === "string") await inspectRoleSource(role, path, true);
    } catch (error) {
      roleSourceErrors = {
        ...roleSourceErrors,
        [role]: sourceOperationErrorMessage(error),
      };
    } finally {
      busy = false;
    }
  }

  function removeRoleSource(role: string) {
    project.sources = project.sources.filter((source) => source.role !== role);
    const nextInfo = { ...roleSourceInfo };
    const nextErrors = { ...roleSourceErrors };
    delete nextInfo[role];
    delete nextErrors[role];
    roleSourceInfo = nextInfo;
    roleSourceErrors = nextErrors;
    markDirty();
    operationStatus = `${role} source removed`;
    void refreshOutputs();
  }

  function managedSourceRoles(): AdditionalSourceRoleDependency[] {
    const dependencies = additionalSourceRoleDependencies(
      availablePresets,
      project.selectedPresets.map((preset) => preset.id),
    );
    const roles = Object.fromEntries(
      dependencies.map((dependency) => [dependency.role, dependency]),
    );
    for (const source of project.sources) {
      if (source.role === "master" || roles[source.role]) continue;
      roles[source.role] = {
        role: source.role,
        required: false,
        description: "Configured project source role.",
        outputTypes: [],
      };
    }
    return Object.values(roles).sort((left, right) => left.role.localeCompare(right.role));
  }

  async function refreshPreview() {
    busy = true;
    try {
      await renderSourcePreview();
    } finally {
      busy = false;
    }
  }

  function removeMasterSource() {
    project.sources = project.sources.filter((source) => source.role !== "master");
    clearSourceUi();
    markDirty();
    operationStatus = "Master source removed";
    void refreshOutputs();
  }

  function previewDisplayWidth(): number {
    return preview ? (preview.width * previewZoom) / displayPixelRatio : 0;
  }

  function previewDisplayHeight(): number {
    return preview ? (preview.height * previewZoom) / displayPixelRatio : 0;
  }

  function addCustomOutput() {
    const output = nextCustomOutput(project.customOutputs, project.baseFilename);
    project.customOutputs = [...project.customOutputs, output];
    project.enabledOutputs = [...new Set([...project.enabledOutputs, output.id])];
    markDirty();
    operationStatus = "Custom output added";
    void refreshOutputs().then(async () => {
      await tick();
      outputTable?.scrollToOutput(output.id);
    });
  }

  function toggleCustomOutput(id: string, included: boolean) {
    project.enabledOutputs = included
      ? project.enabledOutputs.includes(id)
        ? project.enabledOutputs
        : [...project.enabledOutputs, id]
      : project.enabledOutputs.filter((outputId) => outputId !== id);
    markDirty();
    void refreshOutputs();
  }

  function togglePreset(preset: PresetSummary, selected: boolean) {
    const conflictingPresets = preset.exclusiveGroup
      ? availablePresets.filter(
          (candidate) =>
            candidate.id !== preset.id && candidate.exclusiveGroup === preset.exclusiveGroup,
        )
      : [];
    const conflictingIds = new Set(conflictingPresets.map((candidate) => candidate.id));
    const conflictingOutputIds = new Set(
      conflictingPresets.flatMap((candidate) => candidate.outputIds),
    );
    project.selectedPresets = selected
      ? project.selectedPresets.some((candidate) => candidate.id === preset.id)
        ? project.selectedPresets
        : [
            ...project.selectedPresets.filter((candidate) => !conflictingIds.has(candidate.id)),
            { id: preset.id, version: preset.version },
          ]
      : project.selectedPresets.filter((candidate) => candidate.id !== preset.id);
    project.enabledOutputs = selected
      ? [
          ...new Set([
            ...project.enabledOutputs.filter((outputId) => !conflictingOutputIds.has(outputId)),
            ...preset.defaultOutputIds,
          ]),
        ]
      : project.enabledOutputs.filter((outputId) => !preset.outputIds.includes(outputId));
    markDirty();
    operationStatus = `${preset.displayName} ${selected ? "enabled" : "disabled"}`;
    void refreshOutputs();
    void refreshPresetUpgrades();
  }

  function toggleOptionalGroup(group: PresetOptionalOutputGroup, enabled: boolean) {
    const outputIds = group.outputs ?? [];
    project.enabledOutputs = enabled
      ? [...new Set([...project.enabledOutputs, ...outputIds])]
      : project.enabledOutputs.filter((outputId) => !outputIds.includes(outputId));
    markDirty();
    operationStatus = `${group.displayName} ${enabled ? "enabled" : "disabled"}`;
    void refreshOutputs();
  }

  function selectedOptionalGroups(): Array<{
    presetId: string;
    presetName: string;
    group: PresetOptionalOutputGroup;
  }> {
    const selectedIds = new Set(project.selectedPresets.map((preset) => preset.id));
    return availablePresets
      .filter((preset) => selectedIds.has(preset.id))
      .flatMap((preset) =>
        preset.optionalOutputGroups.map((group) => ({
          presetId: preset.id,
          presetName: preset.displayName,
          group,
        })),
      );
  }

  function setCombinedZip(enabled: boolean) {
    project.exportSettings = {
      ...project.exportSettings,
      createCombinedZip: enabled,
    };
    markDirty();
  }

  function updateOutput(id: string, field: CustomOutputField, rawValue: string | number) {
    let value: string | number = rawValue;
    if (field === "width" || field === "height") {
      value = Math.max(1, Math.min(16_384, Math.round(Number(rawValue) || 1)));
    } else {
      value = String(rawValue);
      if ((field === "filename" || field === "usage") && !value.trim()) {
        outputError = `${field === "filename" ? "Filename" : "Usage"} cannot be empty.`;
        return;
      }
    }

    const resolved = outputs.find((output) => output.id === id);
    if (resolved?.presetId) {
      if (field !== "width" && field !== "height") return;
      const current = project.outputOverrides[id] ?? emptyOutputOverride();
      project.outputOverrides = {
        ...project.outputOverrides,
        [id]: {
          ...current,
          [field]: value,
        },
      };
    } else {
      project.customOutputs = project.customOutputs.map((output) =>
        output.id === id ? { ...output, [field]: value } : output,
      );
      if ((field === "width" || field === "height") && project.outputOverrides[id]) {
        project.outputOverrides = {
          ...project.outputOverrides,
          [id]: {
            ...project.outputOverrides[id],
            [field]: null,
          },
        };
      }
    }
    markDirty();
    void refreshOutputs();
  }

  function removeCustomOutput(id: string) {
    project.customOutputs = project.customOutputs.filter((output) => output.id !== id);
    project.enabledOutputs = project.enabledOutputs.filter((outputId) => outputId !== id);
    const remainingOverrides = { ...project.outputOverrides };
    delete remainingOverrides[id];
    project.outputOverrides = remainingOverrides;
    markDirty();
    operationStatus = "Custom output removed";
    void refreshOutputs();
  }

  async function replaceOutputSource(id: string) {
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Icon source", extensions: ["svg", "png"] }],
      });
      if (typeof path !== "string") return;
      const info = await invoke<SourceInfo>("import_source", { path, role: "override" });
      const current = project.outputOverrides[id] ?? emptyOutputOverride();
      project.outputOverrides = {
        ...project.outputOverrides,
        [id]: {
          ...current,
          sourceRole: null,
          sourcePath: info.path,
          sourceContentHash: info.contentHash,
        },
      };
      markDirty();
      operationStatus = `Source replaced for ${id}`;
      await refreshOutputs();
    } catch (error) {
      outputError = sourceOperationErrorMessage(error);
    }
  }

  function resetOutputSource(id: string) {
    const current = project.outputOverrides[id] ?? emptyOutputOverride();
    project.outputOverrides = {
      ...project.outputOverrides,
      [id]: {
        ...current,
        sourceRole: null,
        sourcePath: null,
        sourceContentHash: null,
      },
    };
    markDirty();
    operationStatus = `Source reset for ${id}`;
    void refreshOutputs();
  }

  function setOutputUpscale(id: string, allowed: boolean) {
    const current = project.outputOverrides[id] ?? emptyOutputOverride();
    project.outputOverrides = {
      ...project.outputOverrides,
      [id]: { ...current, allowUpscale: allowed },
    };
    markDirty();
    void refreshOutputs();
  }

  async function createExport() {
    exportError = "";
    exportReport = null;
    await refreshOutputs();
    const included = outputs.filter((output) => output.included);
    if (!included.length || included.some((output) => output.status === "error")) {
      exportError = included.length
        ? "Resolve every included output error before creating the package."
        : "Enable at least one custom output before creating the package.";
      return;
    }

    try {
      const parent = await open({ multiple: false, directory: true });
      if (typeof parent !== "string") {
        operationStatus = "Export cancelled";
        return;
      }
      const inspection = await invoke<DestinationInspection>("inspect_export_destination", {
        parent,
        baseFilename: project.baseFilename,
      });
      if (inspection.state === "non-empty-directory") {
        exportError = `The destination ${inspection.path} already contains files that are not an Imaguncula package, so it is never replaced. Choose a different parent directory or remove it manually.`;
        operationStatus = "Export cancelled";
        return;
      }
      let overwrite = false;
      if (inspection.requiresConfirmation) {
        overwrite = window.confirm(
          `Replace the existing export at:\n\n${inspection.path}\n\nThe old package is retained unless the new package verifies successfully.`,
        );
        if (!overwrite) {
          operationStatus = "Export cancelled";
          return;
        }
      }
      project.exportSettings = {
        ...project.exportSettings,
        overwriteConfirmed: overwrite,
      };
      markDirty();
      exportProgress = { jobId: "", completed: 0, total: included.length, currentOutput: null };
      pendingExportStart = true;
      const startedJob = await invoke<string>("start_export", {
        project,
        parent,
        overwrite,
      });
      if (pendingExportStart) {
        pendingExportStart = false;
        activeExportJob = startedJob;
        exportProgress = { ...exportProgress, jobId: startedJob };
        operationStatus = "Creating output package";
      }
    } catch (error) {
      pendingExportStart = false;
      activeExportJob = null;
      exportProgress = null;
      exportError = exportErrorMessage(error);
    }
  }

  async function cancelActiveExport() {
    if (!activeExportJob) return;
    try {
      await invoke<boolean>("cancel_export", { jobId: activeExportJob });
      operationStatus = "Cancelling export";
    } catch (error) {
      exportError = exportErrorMessage(error);
    }
  }

  async function createProject() {
    if (!canDiscardChanges()) return;

    busy = true;
    operationError = "";
    try {
      project = await invoke<ProjectState>("new_project", { projectName: "Untitled project" });
      clearSourceUi();
      currentPath = null;
      dirty = false;
      operationStatus = "New unsaved project";
      outputs = [];
      outputError = "";
      exportReport = null;
      await refreshOutputs();
      await refreshPresetUpgrades();
    } catch (error) {
      operationError = projectErrorMessage(error);
    } finally {
      busy = false;
    }
  }

  async function openProject() {
    if (!canDiscardChanges()) return;

    busy = true;
    operationError = "";
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "Imaguncula project", extensions: ["json"] }],
      });
      if (typeof path !== "string") {
        operationStatus = "Open cancelled";
        return;
      }

      const loaded = await invoke<ProjectLoad>("load_project", { path });
      project = loaded.project;
      currentPath = path;
      dirty = loaded.migrationNotices.length > 0;
      if (loaded.externalSources.length) {
        const list = loaded.externalSources
          .map((external) => `${external.reference}: ${external.path}`)
          .join("\n");
        const importExternal = window.confirm(
          `This project references ${loaded.externalSources.length} file(s) outside its own folder:\n\n${list}\n\nImport these files? Choosing Cancel removes the references so you can relink sources you trust.`,
        );
        if (!importExternal) {
          const externalPaths = new Set(loaded.externalSources.map((external) => external.path));
          project.sources = project.sources.filter((source) => !externalPaths.has(source.path));
          for (const override of Object.values(project.outputOverrides)) {
            if (override.sourcePath && externalPaths.has(override.sourcePath)) {
              override.sourcePath = null;
              override.sourceContentHash = null;
            }
          }
          dirty = true;
        }
      }
      const openNotices: string[] = [];
      if (loaded.migrationNotices.length) {
        openNotices.push(`${loaded.migrationNotices.length} migration notice(s)`);
      }
      if (project.acceptedWarnings.length) {
        openNotices.push(`${project.acceptedWarnings.length} pre-accepted warning(s)`);
      }
      const preAuthorizedUpscales = Object.values(project.outputOverrides).filter(
        (override) => override.allowUpscale,
      ).length;
      if (preAuthorizedUpscales) {
        openNotices.push(`${preAuthorizedUpscales} pre-authorized upscale(s)`);
      }
      operationStatus = openNotices.length
        ? `Project opened with ${openNotices.join(", ")}`
        : "Project opened";
      clearSourceUi();
      const masterSource = project.sources.find((source) => source.role === "master");
      if (masterSource) {
        await importMasterSource(masterSource.path, false, masterSource.contentHash);
      }
      for (const source of project.sources.filter((source) => source.role !== "master")) {
        try {
          await inspectRoleSource(source.role, source.path, false, source.contentHash);
        } catch (error) {
          roleSourceErrors = {
            ...roleSourceErrors,
            [source.role]: sourceOperationErrorMessage(error),
          };
        }
      }
      await refreshOutputs();
      await refreshPresetUpgrades();
    } catch (error) {
      operationError = projectErrorMessage(error);
    } finally {
      busy = false;
    }
  }

  function withProjectExtension(path: string): string {
    if (path.toLocaleLowerCase().endsWith(".imaguncula.json")) return path;
    if (path.toLocaleLowerCase().endsWith(".json")) return `${path.slice(0, -5)}.imaguncula.json`;
    return `${path}.imaguncula.json`;
  }

  async function saveProject() {
    busy = true;
    operationError = "";
    try {
      let path = currentPath;
      if (!path) {
        const selectedPath = await save({
          defaultPath: `${project.baseFilename}.imaguncula.json`,
          filters: [{ name: "Imaguncula project", extensions: ["json"] }],
        });
        if (typeof selectedPath !== "string") {
          operationStatus = "Save cancelled";
          return;
        }
        path = withProjectExtension(selectedPath);
      }

      await invoke("save_project", { path, project });
      currentPath = path;
      dirty = false;
      operationStatus = "Project saved";
    } catch (error) {
      operationError = projectErrorMessage(error);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:head>
  <title>{dirty ? "• " : ""}{project.projectName} — Imaguncula</title>
</svelte:head>

<header class="app-header">
  <div class="brand">
    <img class="brand-mark" src="/favicon.svg" alt="" />
    <div>
      <p class="eyebrow">Offline icon asset generator</p>
      <h1>Imaguncula</h1>
    </div>
  </div>
  <nav aria-label="Project actions">
    <button type="button" onclick={createProject} disabled={busy}>New</button>
    <button type="button" onclick={openProject} disabled={busy}>Open</button>
    <button class="primary" type="button" onclick={saveProject} disabled={busy}>Save</button>
  </nav>
</header>

<main>
  <section class="workspace" aria-labelledby="project-heading">
    <div class="section-heading">
      <div>
        <p class="section-label">Project file · schema v{project.schemaVersion}</p>
        <h2 id="project-heading">Project identity</h2>
      </div>
      <p class="path" title={currentPath ?? "Not saved"}>
        {currentPath ?? "Not saved"}
      </p>
    </div>

    <div class="fields">
      <label>
        <span>Project name</span>
        <input bind:value={project.projectName} oninput={markDirty} maxlength="200" />
      </label>
      <label>
        <span>Base filename</span>
        <input bind:value={project.baseFilename} oninput={markDirty} maxlength="120" />
        <small>Used as the generated asset filename stem.</small>
      </label>
    </div>

    <dl class="project-summary">
      <div>
        <dt>Sources</dt>
        <dd>{project.sources.length}</dd>
      </div>
      <div>
        <dt>Presets</dt>
        <dd>{project.selectedPresets.length}</dd>
      </div>
      <div>
        <dt>Custom outputs</dt>
        <dd>{project.customOutputs.length}</dd>
      </div>
      <div>
        <dt>Accepted warnings</dt>
        <dd>{project.acceptedWarnings.length}</dd>
      </div>
      <div>
        <dt>Preset migrations</dt>
        <dd>{project.presetMigrations.length}</dd>
      </div>
    </dl>
  </section>

  {#if presetUpgrades.length}
    <section class="workspace migration-workspace" aria-labelledby="preset-migration-heading">
      <div class="section-heading">
        <div>
          <p class="section-label">Review required · project remains pinned until accepted</p>
          <h2 id="preset-migration-heading">Preset updates</h2>
        </div>
      </div>
      <p class="section-intro">
        Review output identity changes before changing the version stored in this project. Accepting
        records the exact comparison fingerprint and marks the project unsaved.
      </p>
      <div class="migration-list">
        {#each presetUpgrades as upgrade (`${upgrade.diff.presetId}:${upgrade.diff.toVersion}`)}
          <article class="migration-card">
            <div class="migration-card-heading">
              <div>
                <h3>{upgrade.diff.presetId}</h3>
                <p>Version {upgrade.diff.fromVersion} → {upgrade.diff.toVersion}</p>
              </div>
              <button
                class="primary"
                type="button"
                disabled={busy}
                onclick={() => acceptPresetUpgrade(upgrade)}
              >
                Accept and migrate
              </button>
            </div>
            <div class="migration-diff">
              <div>
                <h4>Added ({upgrade.diff.addedOutputs.length})</h4>
                {#if upgrade.diff.addedOutputs.length}
                  <ul>
                    {#each upgrade.diff.addedOutputs as outputId (outputId)}
                      <li><code>{outputId}</code></li>
                    {/each}
                  </ul>
                {:else}
                  <p>None</p>
                {/if}
              </div>
              <div>
                <h4>Removed ({upgrade.diff.removedOutputs.length})</h4>
                {#if upgrade.diff.removedOutputs.length}
                  <ul>
                    {#each upgrade.diff.removedOutputs as outputId (outputId)}
                      <li><code>{outputId}</code></li>
                    {/each}
                  </ul>
                {:else}
                  <p>None</p>
                {/if}
              </div>
              <div>
                <h4>Renamed ({upgrade.diff.renamedOutputs.length})</h4>
                {#if upgrade.diff.renamedOutputs.length}
                  <ul>
                    {#each upgrade.diff.renamedOutputs as rename (`${rename.from}:${rename.to}`)}
                      <li><code>{rename.from}</code> → <code>{rename.to}</code></li>
                    {/each}
                  </ul>
                {:else}
                  <p>None</p>
                {/if}
              </div>
            </div>
            <p class="migration-fingerprint" title={upgrade.diffFingerprint}>
              Review fingerprint: <code>{upgrade.diffFingerprint}</code>
            </p>
          </article>
        {/each}
      </div>
    </section>
  {/if}

  <section
    class="workspace source-workspace"
    class:drag-active={dragActive}
    aria-labelledby="source-heading"
  >
    <div class="section-heading">
      <div>
        <p class="section-label">Original artwork · local processing only</p>
        <h2 id="source-heading">Master source</h2>
      </div>
      {#if sourceInfo}
        <div class="source-actions">
          <button type="button" onclick={chooseMasterSource} disabled={busy}>Replace</button>
          <button type="button" onclick={removeMasterSource} disabled={busy}>Remove</button>
        </div>
      {/if}
    </div>

    {#if dragActive}
      <p class="drop-indicator" aria-live="polite">Release to inspect this source</p>
    {/if}

    {#if sourceInfo}
      <article class="source-card">
        <div class="source-title">
          <div>
            <p class="source-format">{sourceInfo.format.toUpperCase()} · {sourceInfo.shape}</p>
            <h3>{sourceFilename(sourceInfo.path)}</h3>
          </div>
          <p class="hash" title={sourceInfo.contentHash}>{sourceInfo.contentHash.slice(0, 12)}…</p>
        </div>
        <p class="source-path" title={sourceInfo.path}>{sourceInfo.path}</p>

        <dl class="source-facts">
          <div>
            <dt>Dimensions</dt>
            <dd>{sourceInfo.dimensions.width}×{sourceInfo.dimensions.height}</dd>
          </div>
          <div>
            <dt>File size</dt>
            <dd>{Math.max(1, Math.round(sourceInfo.byteLength / 1024))} KiB</dd>
          </div>
          <div>
            <dt>Transparency</dt>
            <dd>
              {sourceInfo.png
                ? sourceInfo.png.hasTransparentPixels
                  ? "Present"
                  : sourceInfo.png.hasAlphaChannel
                    ? "Opaque alpha"
                    : "No alpha"
                : "Preserved"}
            </dd>
          </div>
          <div>
            <dt>Color profile</dt>
            <dd>{sourceInfo.png?.hasIccProfile ? "ICC detected" : "sRGB/default"}</dd>
          </div>
        </dl>

        {#if sourceInfo.warnings.length}
          <div class="warning-list" aria-labelledby="source-warnings-heading">
            <h4 id="source-warnings-heading">Source warnings</h4>
            <ul>
              {#each sourceInfo.warnings as warning (JSON.stringify(warning))}
                <li>{sourceWarningMessage(warning)}</li>
              {/each}
            </ul>
          </div>
        {:else}
          <p class="ready-state">No source warnings detected.</p>
        {/if}
      </article>

      <div class="preview-workspace">
        <div class="preview-controls">
          <div>
            <p class="section-label">Direct from original</p>
            <h3>Pixel preview</h3>
          </div>
          <div class="preview-fields">
            <label>
              <span>Width</span>
              <input type="number" min="1" max="4096" bind:value={previewWidth} />
            </label>
            <label>
              <span>Height</span>
              <input type="number" min="1" max="4096" bind:value={previewHeight} />
            </label>
            <label>
              <span>Zoom</span>
              <select bind:value={previewZoom}>
                <option value={1}>Actual</option>
                <option value={2}>2×</option>
                <option value={4}>4×</option>
                <option value={8}>8×</option>
                <option value={16}>16×</option>
              </select>
            </label>
            <label>
              <span>Background</span>
              <select bind:value={previewBackground}>
                <option value="checkerboard">Checkerboard</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
                <option value="transparent">Transparent</option>
              </select>
            </label>
            <button class="primary" type="button" onclick={refreshPreview} disabled={busy}>
              Render
            </button>
          </div>
        </div>

        <div class="preview-frame {previewBackground}" aria-label="Rendered source preview">
          {#if preview && previewUrl}
            <img
              src={previewUrl}
              alt={`Rendered preview of ${sourceFilename(sourceInfo.path)}`}
              width={preview.width}
              height={preview.height}
              draggable="false"
              class:pixelated={previewZoom > 1}
              style={`width: ${previewDisplayWidth()}px; height: ${previewDisplayHeight()}px`}
            />
          {:else}
            <p>Preview unavailable</p>
          {/if}
        </div>
        {#if preview}
          <p class="preview-caption">
            {preview.width}×{preview.height} output pixels · {previewZoom}× inspection ·
            {displayPixelRatio.toFixed(1)} device pixel ratio{preview.cacheHit ? " · cached" : ""}
          </p>
        {/if}
      </div>
    {:else}
      <button class="drop-zone" type="button" onclick={chooseMasterSource} disabled={busy}>
        <strong>Select an SVG or PNG</strong>
        <span>or drag one onto this window</span>
        <small>
          {desktopRuntime
            ? "SVG is preferred. PNG files are decoded under dimension, pixel, and memory limits."
            : "Frontend preview only. Run pnpm tauri dev to select and process local files."}
        </small>
      </button>
    {/if}

    {#if sourceError}
      <p class="source-error" role="alert">{sourceError}</p>
    {/if}
  </section>

  {#if managedSourceRoles().length}
    <section class="workspace" aria-labelledby="role-sources-heading">
      <div class="section-heading">
        <div>
          <p class="section-label">Preset-specific artwork · validated independently</p>
          <h2 id="role-sources-heading">Additional source roles</h2>
        </div>
      </div>
      <p class="section-intro">
        Selected layered or adaptive presets declare these roles. Required roles block export until
        a valid local SVG or PNG is assigned.
      </p>
      <div class="role-source-grid">
        {#each managedSourceRoles() as role (role.role)}
          {@const configured = project.sources.find((source) => source.role === role.role)}
          {@const info = roleSourceInfo[role.role]}
          <article class="role-source-card">
            <div class="source-title">
              <div>
                <p class="source-format">{role.required ? "Required" : "Optional"} source role</p>
                <h3>{role.role}</h3>
              </div>
              <div class="source-actions">
                <button type="button" onclick={() => chooseRoleSource(role.role)} disabled={busy}>
                  {configured ? "Replace" : "Select"}
                </button>
                {#if configured}
                  <button type="button" onclick={() => removeRoleSource(role.role)} disabled={busy}>
                    Remove
                  </button>
                {/if}
              </div>
            </div>
            <p>{role.description}</p>
            {#if role.outputTypes.length}
              <div class="role-output-types">
                <p>Selected output types</p>
                <ul>
                  {#each role.outputTypes as outputType (outputType.presetId)}
                    <li>
                      <span>{outputType.required ? "Required for" : "Optional for"}</span>
                      <strong>{outputType.displayName}</strong>
                    </li>
                  {/each}
                </ul>
              </div>
            {/if}
            {#if configured}
              <p class="source-path" title={configured.path}>{configured.path}</p>
            {:else}
              <p class="missing-source">No source assigned.</p>
            {/if}
            {#if info}
              <dl class="role-source-facts">
                <div>
                  <dt>Format</dt>
                  <dd>{info.format.toUpperCase()}</dd>
                </div>
                <div>
                  <dt>Dimensions</dt>
                  <dd>{info.dimensions.width}×{info.dimensions.height}</dd>
                </div>
                <div>
                  <dt>Transparency</dt>
                  <dd>{info.png?.hasTransparentPixels ? "Present" : "None detected"}</dd>
                </div>
              </dl>
            {/if}
            {#if roleSourceErrors[role.role]}
              <p class="source-error" role="alert">{roleSourceErrors[role.role]}</p>
            {/if}
          </article>
        {/each}
      </div>
    </section>
  {/if}

  <section class="workspace output-workspace" aria-labelledby="outputs-heading">
    <div class="section-heading">
      <div>
        <p class="section-label">Reviewed before every write</p>
        <h2 id="outputs-heading">Output review</h2>
      </div>
      <button
        type="button"
        onclick={addCustomOutput}
        disabled={outputBusy || activeExportJob !== null}
      >
        Add custom output
      </button>
    </div>

    <p class="section-intro">
      Every included row is rendered independently from its selected source. Package paths and
      dimensions are verified in staging before the final directory is replaced.
    </p>

    {#if availablePresets.length}
      <fieldset class="preset-picker">
        <legend>Platform presets</legend>
        {#each availablePresets as preset (preset.id)}
          {@const sourceNotice = additionalSourceRoleNotice(preset)}
          <label class="preset-option">
            <input
              type="checkbox"
              checked={project.selectedPresets.some((selected) => selected.id === preset.id)}
              disabled={activeExportJob !== null}
              onchange={(event) => togglePreset(preset, event.currentTarget.checked)}
            />
            <span>
              <strong>{preset.displayName}</strong>
              <small>
                {preset.platform} · version {preset.version} · {preset.outputIds.length} outputs
              </small>
              {#if sourceNotice}
                <small class="preset-source-requirement">{sourceNotice}</small>
              {/if}
            </span>
          </label>
        {/each}
      </fieldset>
      {#if selectedOptionalGroups().length}
        <fieldset class="preset-picker optional-group-picker">
          <legend>Optional preset groups</legend>
          {#each selectedOptionalGroups() as entry (`${entry.presetId}:${entry.group.id}`)}
            {@const outputIds = entry.group.outputs ?? []}
            <label class="preset-option">
              <input
                type="checkbox"
                checked={outputIds.length > 0 &&
                  outputIds.every((outputId) => project.enabledOutputs.includes(outputId))}
                disabled={activeExportJob !== null || outputIds.length === 0}
                onchange={(event) => toggleOptionalGroup(entry.group, event.currentTarget.checked)}
              />
              <span>
                <strong>{entry.group.displayName}</strong>
                <small>{entry.presetName} · {entry.group.description}</small>
              </span>
            </label>
          {/each}
        </fieldset>
      {/if}
    {/if}

    {#if outputs.length}
      <OutputTable
        bind:this={outputTable}
        {outputs}
        onToggle={toggleCustomOutput}
        onUpdate={updateOutput}
        onRemove={removeCustomOutput}
        onReplace={replaceOutputSource}
        onResetSource={resetOutputSource}
        onAllowUpscale={setOutputUpscale}
      />
    {:else}
      <div class="empty-outputs">
        <h3>No outputs selected</h3>
        <p>Enable a platform preset or add a row for an exact custom PNG.</p>
        <button type="button" onclick={addCustomOutput} disabled={outputBusy}>
          Add first output
        </button>
      </div>
    {/if}

    {#if outputError}
      <p class="output-error" role="alert">{outputError}</p>
    {/if}

    <div class="export-panel">
      <div>
        <p class="section-label">Transactional package</p>
        <h3>Create export</h3>
        <p>
          The selected parent receives <code>{project.baseFilename}-icons</code> only after every
          output and <code>report.md</code> verify successfully.
        </p>
      </div>
      <div class="export-actions">
        <label class="checkbox-label">
          <input
            type="checkbox"
            checked={project.exportSettings.createCombinedZip}
            disabled={activeExportJob !== null}
            onchange={(event) => setCombinedZip(event.currentTarget.checked)}
          />
          <span>Include combined ZIP</span>
        </label>
        {#if activeExportJob}
          <button class="danger" type="button" onclick={cancelActiveExport}>Cancel export</button>
        {:else}
          <button
            class="primary"
            type="button"
            onclick={createExport}
            disabled={outputBusy || !outputs.some((output) => output.included)}
          >
            Create
          </button>
        {/if}
      </div>
    </div>

    {#if exportProgress}
      <div class="export-progress" aria-live="polite">
        <progress max={Math.max(1, exportProgress.total)} value={exportProgress.completed}
        ></progress>
        <p>
          {exportProgress.completed} of {exportProgress.total}
          {exportProgress.currentOutput ? ` · ${exportProgress.currentOutput}` : " · preparing"}
        </p>
      </div>
    {/if}

    {#if exportError}
      <p class="output-error" role="alert">{exportError}</p>
    {/if}

    {#if exportReport}
      <article class="export-report" aria-labelledby="export-report-heading">
        <div class="section-heading">
          <div>
            <p class="section-label">Verified package</p>
            <h3 id="export-report-heading">Generation report</h3>
          </div>
          <p class="path" title={exportReport.destination}>{exportReport.destination}</p>
        </div>
        <dl class="report-summary">
          <div>
            <dt>Files</dt>
            <dd>{exportReport.report.files.length}</dd>
          </div>
          <div>
            <dt>Warnings</dt>
            <dd>{exportReport.report.warnings.length}</dd>
          </div>
          <div>
            <dt>Outstanding</dt>
            <dd>{exportReport.report.warnings.filter((warning) => !warning.accepted).length}</dd>
          </div>
        </dl>
        <details class="report-details">
          <summary>Show report</summary>
          <div class="rendered-report">
            <h4>Imaguncula generation report</h4>
            <dl class="report-identity">
              <div>
                <dt>Project</dt>
                <dd>{exportReport.report.projectName}</dd>
              </div>
              <div>
                <dt>Base filename</dt>
                <dd><code>{exportReport.report.baseFilename}</code></dd>
              </div>
            </dl>

            <h5>Generated files</h5>
            <div class="report-files" role="region" aria-label="Generated files">
              <table>
                <thead>
                  <tr>
                    <th scope="col">Output</th>
                    <th scope="col">Path</th>
                    <th scope="col">Dimensions</th>
                    <th scope="col">Format</th>
                    <th scope="col">Source</th>
                  </tr>
                </thead>
                <tbody>
                  {#each exportReport.report.files as file (`${file.outputId}:${file.path}`)}
                    <tr>
                      <td>{file.outputId}</td>
                      <td><code>{file.path}</code></td>
                      <td>{reportDimensions(file)}</td>
                      <td>{file.format.toUpperCase()}</td>
                      <td>
                        <span>{file.sourceLabel}</span>
                        <code title={file.sourcePath}>{file.sourcePath}</code>
                      </td>
                    </tr>
                  {:else}
                    <tr>
                      <td colspan="5">No generated files.</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>

            <h5>Warnings</h5>
            {#if exportReport.report.warnings.length}
              <ul class="report-warnings">
                {#each exportReport.report.warnings as warning (`${warning.outputId}:${warning.code}`)}
                  <li>
                    <span class:accepted={warning.accepted} class:outstanding={!warning.accepted}>
                      {warning.accepted ? "Accepted" : "Outstanding"}
                    </span>
                    <strong>{warning.outputId}</strong>
                    <code>{warning.code}</code>
                    <span>{warning.message}</span>
                  </li>
                {/each}
              </ul>
            {:else}
              <p>No warnings.</p>
            {/if}
          </div>
        </details>
      </article>
    {/if}
  </section>
</main>

<footer>
  <p class="status" aria-live="polite">{busy ? "Working" : operationStatus}</p>
  <p class="core-status">{shellStatus}</p>
  {#if operationError}
    <p class="error" role="alert">{operationError}</p>
  {/if}
</footer>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(:root) {
    font-family:
      Inter,
      ui-sans-serif,
      system-ui,
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      sans-serif;
    color: #18201d;
    background: #eef1e9;
    font-synthesis: none;
    text-rendering: optimizeLegibility;
  }

  :global(body) {
    margin: 0;
    min-width: 320px;
    min-height: 100vh;
    background:
      radial-gradient(circle at 88% 14%, rgba(183, 196, 151, 0.55), transparent 31rem),
      linear-gradient(145deg, #f7f4ea 0%, #edf1e8 48%, #e0e8dc 100%);
  }

  :global(button),
  :global(input),
  :global(select) {
    font: inherit;
  }

  :global(:focus-visible) {
    outline: 3px solid #176b48;
    outline-offset: 3px;
  }

  .app-header,
  main,
  footer {
    width: min(72rem, calc(100% - 3rem));
    margin-right: auto;
    margin-left: auto;
  }

  .app-header {
    padding-top: clamp(2rem, 5vw, 4rem);
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 1.1rem;
  }

  .brand-mark {
    width: 3.4rem;
    height: 3.4rem;
  }

  .app-header,
  .section-heading,
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 2rem;
  }

  .eyebrow,
  .section-label {
    margin: 0 0 0.45rem;
    color: #4e654c;
    font-size: 0.74rem;
    font-weight: 750;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  h1,
  h2,
  h3,
  h4 {
    margin: 0;
    color: #17251f;
    font-family: Georgia, "Times New Roman", serif;
    font-weight: 500;
    letter-spacing: -0.045em;
  }

  h1 {
    font-size: clamp(2.7rem, 7vw, 4.8rem);
    line-height: 0.9;
  }

  h2 {
    font-size: clamp(1.8rem, 4vw, 2.7rem);
  }

  h3 {
    margin: 0;
    font-size: 1.35rem;
  }

  h4 {
    margin: 0;
    font-family: inherit;
    font-size: 0.85rem;
    font-weight: 800;
    letter-spacing: 0;
  }

  nav {
    display: flex;
    flex-wrap: wrap;
    gap: 0.65rem;
  }

  button {
    border: 1px solid #819081;
    border-radius: 0.55rem;
    padding: 0.65rem 1rem;
    background: rgba(255, 255, 255, 0.65);
    color: #203128;
    cursor: pointer;
    font-weight: 700;
  }

  button:hover:not(:disabled) {
    background: #ffffff;
  }

  button.primary {
    border-color: #234f39;
    background: #234f39;
    color: #ffffff;
  }

  button.primary:hover:not(:disabled) {
    background: #183b2a;
  }

  button.danger {
    border-color: #9d5d5d;
    background: #fff4f2;
    color: #7b2929;
  }

  button:disabled {
    cursor: wait;
    opacity: 0.55;
  }

  .workspace {
    margin-top: clamp(3rem, 8vw, 6rem);
    border: 1px solid rgba(59, 76, 65, 0.25);
    border-radius: 1rem;
    padding: clamp(1.4rem, 4vw, 2.5rem);
    background: rgba(255, 255, 255, 0.58);
    box-shadow: 0 1.25rem 4rem rgba(49, 64, 53, 0.08);
  }

  .path {
    max-width: 46%;
    margin: 0;
    overflow: hidden;
    color: #657169;
    font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
    font-size: 0.78rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1.25rem;
    margin-top: 2.25rem;
  }

  label {
    display: grid;
    gap: 0.45rem;
  }

  label span {
    color: #33443a;
    font-size: 0.85rem;
    font-weight: 750;
  }

  input {
    width: 100%;
    border: 1px solid #9ba79e;
    border-radius: 0.5rem;
    padding: 0.72rem 0.8rem;
    background: rgba(255, 255, 255, 0.85);
    color: #18201d;
  }

  select {
    min-width: 7rem;
    border: 1px solid #9ba79e;
    border-radius: 0.5rem;
    padding: 0.62rem 0.7rem;
    background: rgba(255, 255, 255, 0.9);
    color: #18201d;
  }

  small {
    color: #667269;
    font-size: 0.76rem;
  }

  .project-summary {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
    gap: 1px;
    margin: 2.25rem 0 0;
    overflow: hidden;
    border: 1px solid rgba(59, 76, 65, 0.18);
    border-radius: 0.65rem;
    background: rgba(59, 76, 65, 0.18);
  }

  .project-summary div {
    padding: 1rem;
    background: rgba(249, 250, 246, 0.92);
  }

  .source-workspace {
    position: relative;
    transition:
      border-color 120ms ease,
      box-shadow 120ms ease;
  }

  .migration-list {
    display: grid;
    gap: 1rem;
  }

  .migration-card {
    border: 1px solid rgba(165, 107, 25, 0.4);
    border-radius: 0.8rem;
    padding: 1.2rem;
    background: #fffaf0;
  }

  .migration-card-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .migration-card-heading p,
  .migration-diff p,
  .migration-fingerprint {
    margin: 0.35rem 0 0;
    color: #5d665f;
    font-size: 0.8rem;
  }

  .migration-diff {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1rem;
    margin-top: 1.2rem;
  }

  .migration-diff ul {
    max-height: 11rem;
    margin: 0.55rem 0 0;
    overflow: auto;
    padding-left: 1.2rem;
  }

  .migration-diff li + li {
    margin-top: 0.3rem;
  }

  .migration-fingerprint {
    overflow-wrap: anywhere;
  }

  .source-workspace.drag-active {
    border-color: #176b48;
    box-shadow: 0 0 0 4px rgba(23, 107, 72, 0.18);
  }

  .role-source-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr));
    gap: 1rem;
  }

  .role-source-card {
    border: 1px solid rgba(59, 76, 65, 0.22);
    border-radius: 0.8rem;
    padding: 1.1rem;
    background: rgba(249, 250, 246, 0.88);
  }

  .role-source-card > p {
    color: #56645b;
    line-height: 1.45;
  }

  .role-output-types {
    margin: 0.9rem 0;
    border-radius: 0.55rem;
    padding: 0.7rem 0.8rem;
    background: #eef2e9;
  }

  .role-output-types > p {
    margin: 0 0 0.45rem;
    color: #536158;
    font-size: 0.7rem;
    font-weight: 850;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .role-output-types ul {
    display: grid;
    gap: 0.35rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .role-output-types li {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 0.75rem;
    color: #33443a;
    font-size: 0.78rem;
  }

  .role-output-types li span {
    color: #5a675f;
  }

  .role-source-facts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
    margin: 1rem 0 0;
  }

  .role-source-facts dd {
    font-family: inherit;
    font-size: 0.9rem;
    font-weight: 750;
  }

  .missing-source {
    border-left: 4px solid #a56b19;
    padding-left: 0.75rem;
    color: #604412;
    font-weight: 700;
  }

  .source-actions,
  .preview-fields {
    display: flex;
    align-items: flex-end;
    flex-wrap: wrap;
    gap: 0.65rem;
  }

  .drop-indicator {
    position: absolute;
    z-index: 2;
    inset: 0.75rem;
    display: grid;
    place-items: center;
    margin: 0;
    border: 2px dashed #176b48;
    border-radius: 0.8rem;
    background: rgba(239, 249, 241, 0.94);
    color: #164d35;
    font-size: 1.1rem;
    font-weight: 800;
    pointer-events: none;
  }

  .drop-zone {
    width: 100%;
    min-height: 13rem;
    margin-top: 2rem;
    border: 2px dashed #819081;
    border-radius: 0.85rem;
    display: grid;
    place-content: center;
    gap: 0.45rem;
    background: rgba(249, 250, 246, 0.62);
    text-align: center;
  }

  .drop-zone strong {
    color: #203128;
    font-family: Georgia, "Times New Roman", serif;
    font-size: 1.45rem;
    font-weight: 500;
  }

  .drop-zone span {
    color: #536159;
    font-weight: 600;
  }

  .drop-zone small {
    max-width: 32rem;
    margin-top: 0.5rem;
    font-weight: 500;
  }

  .source-card {
    margin-top: 2rem;
    border: 1px solid rgba(59, 76, 65, 0.22);
    border-radius: 0.8rem;
    padding: 1.25rem;
    background: rgba(249, 250, 246, 0.88);
  }

  .source-title,
  .preview-controls {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
  }

  .source-format,
  .hash,
  .source-path,
  .preview-caption,
  .ready-state {
    margin: 0;
  }

  .source-format {
    margin-bottom: 0.25rem;
    color: #4e654c;
    font-size: 0.72rem;
    font-weight: 800;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .hash,
  .source-path {
    color: #657169;
    font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
    font-size: 0.74rem;
  }

  .source-path {
    margin-top: 0.75rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .source-facts {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 1rem;
    margin: 1.25rem 0 0;
  }

  .source-facts dd {
    font-family: inherit;
    font-size: 0.95rem;
    font-weight: 750;
  }

  .warning-list,
  .ready-state,
  .source-error {
    margin-top: 1.25rem;
    border-left: 4px solid #a56b19;
    padding: 0.8rem 1rem;
    background: #fff8e7;
    color: #604412;
  }

  .warning-list ul {
    margin: 0.55rem 0 0;
    padding-left: 1.2rem;
  }

  .warning-list li + li {
    margin-top: 0.35rem;
  }

  .ready-state {
    border-left-color: #2c7a50;
    background: #edf8f0;
    color: #24573c;
    font-size: 0.85rem;
    font-weight: 700;
  }

  .source-error {
    border-left-color: #a02f2f;
    background: #fff1ef;
    color: #742323;
  }

  .preview-workspace {
    margin-top: 1.25rem;
    border-top: 1px solid rgba(59, 76, 65, 0.16);
    padding-top: 1.5rem;
  }

  .preview-fields label {
    gap: 0.3rem;
  }

  .preview-fields input {
    width: 5.5rem;
    padding: 0.62rem 0.7rem;
  }

  .preview-frame {
    min-height: 18rem;
    max-height: 38rem;
    margin-top: 1rem;
    overflow: auto;
    border: 1px solid rgba(59, 76, 65, 0.32);
    border-radius: 0.65rem;
    display: grid;
    place-items: center;
    padding: 2rem;
  }

  .preview-frame.checkerboard {
    background-color: #f4f4f0;
    background-image:
      linear-gradient(45deg, #c9cec8 25%, transparent 25%),
      linear-gradient(-45deg, #c9cec8 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #c9cec8 75%),
      linear-gradient(-45deg, transparent 75%, #c9cec8 75%);
    background-position:
      0 0,
      0 8px,
      8px -8px,
      -8px 0;
    background-size: 16px 16px;
  }

  .preview-frame.light {
    background: #ffffff;
  }

  .preview-frame.dark {
    background: #17201c;
    color: #ffffff;
  }

  .preview-frame.transparent {
    background: transparent;
  }

  .preview-frame img {
    max-width: none;
    object-fit: fill;
  }

  .preview-frame img.pixelated {
    image-rendering: pixelated;
  }

  .preview-caption {
    margin-top: 0.65rem;
    color: #5d6a62;
    font-size: 0.78rem;
  }

  .section-intro {
    max-width: 52rem;
    margin: 1rem 0 1.5rem;
    color: #56645b;
    line-height: 1.55;
  }

  .preset-picker {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
    gap: 0.7rem;
    margin: 0 0 1.25rem;
    border: 1px solid rgba(59, 76, 65, 0.2);
    border-radius: 0.7rem;
    padding: 1rem;
  }

  .preset-picker legend {
    padding: 0 0.4rem;
    color: #33443a;
    font-size: 0.8rem;
    font-weight: 800;
  }

  .optional-group-picker {
    margin-top: -0.55rem;
  }

  .preset-option {
    display: flex;
    align-items: flex-start;
    grid-template-columns: none;
    gap: 0.65rem;
    border: 1px solid rgba(59, 76, 65, 0.18);
    border-radius: 0.55rem;
    padding: 0.75rem;
    background: rgba(255, 255, 255, 0.58);
  }

  .preset-option input {
    width: auto;
    margin-top: 0.15rem;
  }

  .preset-option span,
  .preset-option small {
    display: block;
  }

  .preset-option small {
    margin-top: 0.2rem;
  }

  .preset-option small.preset-source-requirement {
    margin-top: 0.55rem;
    border-left: 3px solid #a56b19;
    padding-left: 0.55rem;
    color: #604412;
    font-weight: 750;
    line-height: 1.35;
  }

  .empty-outputs {
    display: grid;
    place-items: center;
    min-height: 12rem;
    border: 2px dashed #9ba79e;
    border-radius: 0.75rem;
    padding: 2rem;
    text-align: center;
  }

  .empty-outputs p {
    max-width: 34rem;
    margin: 0.6rem 0 1rem;
    color: #5d6a62;
  }

  .output-error {
    margin: 1rem 0 0;
    border-left: 4px solid #a02f2f;
    padding: 0.8rem 1rem;
    background: #fff1ef;
    color: #742323;
  }

  .export-panel {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 2rem;
    margin-top: 1.5rem;
    border-top: 1px solid rgba(59, 76, 65, 0.18);
    padding-top: 1.5rem;
  }

  .export-panel p:not(.section-label) {
    max-width: 44rem;
    margin: 0.55rem 0 0;
    color: #5d6a62;
  }

  .export-actions {
    display: grid;
    justify-items: end;
    gap: 0.75rem;
  }

  .export-actions .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    white-space: nowrap;
  }

  code {
    border-radius: 0.25rem;
    padding: 0.08rem 0.25rem;
    background: #e6ebe3;
    font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
    font-size: 0.86em;
  }

  .export-progress {
    margin-top: 1rem;
  }

  .export-progress progress {
    width: 100%;
    height: 0.8rem;
    accent-color: #296a49;
  }

  .export-progress p {
    margin: 0.35rem 0 0;
    color: #415448;
    font-size: 0.78rem;
    font-weight: 700;
  }

  .export-report {
    margin-top: 1.5rem;
    border: 1px solid rgba(44, 122, 80, 0.35);
    border-radius: 0.75rem;
    padding: 1.25rem;
    background: #f1f8f2;
  }

  .report-summary {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1px;
    margin: 1rem 0;
    overflow: hidden;
    border: 1px solid rgba(59, 76, 65, 0.18);
    border-radius: 0.5rem;
    background: rgba(59, 76, 65, 0.18);
  }

  .report-summary div {
    padding: 0.75rem;
    background: #fff;
  }

  .report-summary dd {
    font-size: 1.25rem;
  }

  details summary {
    cursor: pointer;
    color: #294b37;
    font-weight: 750;
  }

  .rendered-report {
    margin-top: 0.8rem;
    border: 1px solid #cad4ca;
    border-radius: 0.45rem;
    padding: 1rem;
    background: #fff;
    color: #27382f;
  }

  .rendered-report h4,
  .rendered-report h5 {
    margin: 0;
    font-family: Georgia, "Times New Roman", serif;
  }

  .rendered-report h4 {
    font-size: 1.25rem;
  }

  .rendered-report h5 {
    margin-top: 1.2rem;
    font-size: 1rem;
  }

  .report-identity {
    display: flex;
    flex-wrap: wrap;
    gap: 1rem 2rem;
    margin: 0.8rem 0 0;
  }

  .report-identity div {
    min-width: min(100%, 12rem);
  }

  .report-identity dd {
    font-family: inherit;
    font-size: 0.9rem;
  }

  .report-files {
    margin-top: 0.55rem;
    overflow-x: auto;
  }

  .report-files table {
    width: 100%;
    min-width: 52rem;
    border-collapse: collapse;
    font-size: 0.76rem;
  }

  .report-files th,
  .report-files td {
    border-bottom: 1px solid #d9e0d8;
    padding: 0.55rem;
    text-align: left;
    vertical-align: top;
  }

  .report-files th {
    background: #eef3ec;
    color: #3b5043;
    font-size: 0.68rem;
    letter-spacing: 0.035em;
    text-transform: uppercase;
  }

  .report-files td code {
    display: block;
    max-width: 24rem;
    overflow-wrap: anywhere;
  }

  .report-files td span + code {
    margin-top: 0.25rem;
    color: #5e6f64;
  }

  .report-warnings {
    display: grid;
    gap: 0.5rem;
    margin: 0.55rem 0 0;
    padding: 0;
    list-style: none;
  }

  .report-warnings li {
    display: grid;
    grid-template-columns: auto auto auto minmax(12rem, 1fr);
    align-items: baseline;
    gap: 0.5rem;
    border: 1px solid #e1d4bb;
    border-radius: 0.4rem;
    padding: 0.55rem;
    background: #fff9eb;
    font-size: 0.76rem;
  }

  .report-warnings .accepted,
  .report-warnings .outstanding {
    border-radius: 999px;
    padding: 0.2rem 0.4rem;
    font-size: 0.64rem;
    font-weight: 850;
    text-transform: uppercase;
  }

  .report-warnings .accepted {
    background: #dff2e5;
    color: #22583a;
  }

  .report-warnings .outstanding {
    background: #f8deda;
    color: #7a2929;
  }

  dt {
    color: #637068;
    font-size: 0.75rem;
  }

  dd {
    margin: 0.2rem 0 0;
    color: #1e3027;
    font-family: Georgia, "Times New Roman", serif;
    font-size: 1.8rem;
  }

  footer {
    position: relative;
    margin-top: 1rem;
    padding-bottom: clamp(2rem, 5vw, 4rem);
    color: #536159;
    font-size: 0.8rem;
  }

  footer p {
    margin: 0;
  }

  .status {
    color: #294b37;
    font-weight: 700;
  }

  .error {
    position: absolute;
    top: calc(100% + 0.75rem);
    left: 0;
    right: 0;
    border-left: 4px solid #a02f2f;
    padding: 0.65rem 0.8rem;
    background: #fff1ef;
    color: #742323;
    line-height: 1.45;
  }

  @media (max-width: 760px) {
    .app-header,
    main,
    footer {
      width: min(100% - 2rem, 42rem);
    }

    .app-header,
    .section-heading,
    footer {
      align-items: flex-start;
      flex-direction: column;
      gap: 1rem;
    }

    .fields {
      grid-template-columns: 1fr;
    }

    .project-summary {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .migration-card-heading {
      align-items: stretch;
      flex-direction: column;
    }

    .migration-diff {
      grid-template-columns: 1fr;
    }

    .source-facts {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .source-title,
    .preview-controls {
      align-items: stretch;
      flex-direction: column;
    }

    .export-panel {
      align-items: stretch;
      flex-direction: column;
      gap: 1rem;
    }

    .report-summary {
      grid-template-columns: 1fr;
    }

    .preview-fields {
      align-items: stretch;
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }

    .preview-fields input,
    .preview-fields select {
      width: 100%;
    }

    .path {
      max-width: 100%;
    }

    .core-status {
      order: 2;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    *,
    *::before,
    *::after {
      scroll-behavior: auto !important;
      transition-duration: 0.01ms !important;
    }
  }
</style>
