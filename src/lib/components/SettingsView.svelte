<script lang="ts">
  import AudioWaveformTrimmer from "./AudioWaveformTrimmer.svelte";
  import {
    MAX_RINGTONE_DURATION_SEC,
    formatFileSize,
  } from "../ringtoneStorage";
  import { settingsStore, type ThemeMode } from "../stores/settings.svelte";

  let fileInputEl = $state<HTMLInputElement | null>(null);
  let trimStartSec = $state(0);
  let trimEndSec = $state(MAX_RINGTONE_DURATION_SEC);

  const THEME_OPTIONS: { label: string; value: ThemeMode }[] = [
    { label: "Dark", value: "dark" },
    { label: "Light", value: "light" },
    { label: "System", value: "system" },
  ];

  const DURATION_OPTIONS = [
    { label: "1 min", ms: 60_000 },
    { label: "5 min", ms: 5 * 60_000 },
    { label: "10 min", ms: 10 * 60_000 },
    { label: "15 min", ms: 15 * 60_000 },
    { label: "25 min", ms: 25 * 60_000 },
    { label: "1 hour", ms: 60 * 60_000 },
  ];

  function formatSec(sec: number): string {
    const safe = Math.max(0, sec);
    const mins = Math.floor(safe / 60);
    const rem = (safe % 60).toFixed(1).padStart(4, "0");
    return `${String(mins).padStart(2, "0")}:${rem}`;
  }

  function handleChooseRingtone(): void {
    settingsStore.clearRingtoneError();
    fileInputEl?.click();
  }

  async function handleFileSelected(event: Event): Promise<void> {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    const result = await settingsStore.uploadCustomRingtone(file);
    if (result === "needs_trim") {
      trimStartSec = 0;
      trimEndSec = Math.min(
        MAX_RINGTONE_DURATION_SEC,
        settingsStore.pendingTrimDurationSec
      );
    }
  }

  function handleTrimRangeChange(nextStart: number, nextEnd: number): void {
    trimStartSec = nextStart;
    trimEndSec = nextEnd;
    settingsStore.stopTrimPreview();
  }
</script>

<section class="settings-page" aria-label="Settings">
  <div class="settings-container">
    <h1 class="page-title">Settings</h1>

    <div class="settings-section">
      <h2 class="section-heading">Appearance</h2>
      <div class="settings-card">
        <div class="setting-row setting-row-col">
          <div class="setting-info">
            <span class="setting-label">Theme</span>
            <span class="setting-desc">Choose between dark, light, or system appearance</span>
          </div>
          <div class="duration-options" role="radiogroup" aria-label="Theme">
            {#each THEME_OPTIONS as opt (opt.value)}
              <button
                type="button"
                role="radio"
                aria-checked={settingsStore.theme === opt.value}
                class="option-btn"
                class:option-btn-active={settingsStore.theme === opt.value}
                onclick={() => settingsStore.setTheme(opt.value)}
              >
                {opt.label}
              </button>
            {/each}
          </div>
        </div>

        <div class="setting-row setting-row-divider">
          <div class="setting-info">
            <span class="setting-label">Transparent window</span>
            <span class="setting-desc">Make the window, header, and sidebar translucent</span>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={settingsStore.transparentWindow}
            aria-label="Transparent window"
            class="toggle-switch"
            class:toggle-switch-on={settingsStore.transparentWindow}
            onclick={() =>
              settingsStore.setTransparentWindow(!settingsStore.transparentWindow)}
          >
            <span class="toggle-thumb"></span>
          </button>
        </div>

        {#if settingsStore.transparentWindow}
          <div class="setting-row setting-row-col setting-row-divider">
            <div class="opacity-header">
              <div class="setting-info">
                <span class="setting-label">Window opacity</span>
                <span class="setting-desc">Adjust background opacity level</span>
              </div>
              <span class="opacity-value">{Math.round(settingsStore.windowOpacity * 100)}%</span>
            </div>
            <input
              type="range"
              min="0.3"
              max="0.95"
              step="0.05"
              value={settingsStore.windowOpacity}
              oninput={(e) =>
                settingsStore.setWindowOpacity(Number((e.currentTarget as HTMLInputElement).value))}
              aria-label="Window opacity"
              class="opacity-slider"
            />
          </div>
        {/if}
      </div>
    </div>

    <div class="settings-section">
      <h2 class="section-heading">Stopwatch</h2>
      <div class="settings-card">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Show tenths of a second</span>
            <span class="setting-desc">Display .d precision on stopwatch cards</span>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={settingsStore.showStopwatchTenths}
            aria-label="Show tenths of a second"
            class="toggle-switch"
            class:toggle-switch-on={settingsStore.showStopwatchTenths}
            onclick={() =>
              settingsStore.setShowStopwatchTenths(!settingsStore.showStopwatchTenths)}
          >
            <span class="toggle-thumb"></span>
          </button>
        </div>

        <div class="setting-row setting-row-divider">
          <div class="setting-info">
            <span class="setting-label">Confirm before resetting</span>
            <span class="setting-desc">Ask for confirmation when resetting a non-zero stopwatch</span>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={settingsStore.confirmStopwatchReset}
            aria-label="Confirm before resetting"
            class="toggle-switch"
            class:toggle-switch-on={settingsStore.confirmStopwatchReset}
            onclick={() =>
              settingsStore.setConfirmStopwatchReset(!settingsStore.confirmStopwatchReset)}
          >
            <span class="toggle-thumb"></span>
          </button>
        </div>
      </div>
    </div>

    <div class="settings-section">
      <h2 class="section-heading">Timer</h2>
      <div class="settings-card">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Completion chime</span>
            <span class="setting-desc">Play a sound when a countdown timer finishes</span>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={settingsStore.timerSoundEnabled}
            aria-label="Completion chime"
            class="toggle-switch"
            class:toggle-switch-on={settingsStore.timerSoundEnabled}
            onclick={() =>
              settingsStore.setTimerSoundEnabled(!settingsStore.timerSoundEnabled)}
          >
            <span class="toggle-thumb"></span>
          </button>
        </div>

        <div class="setting-row setting-row-col setting-row-divider">
          <div class="setting-info">
            <span class="setting-label">Ringtone</span>
            <span class="setting-desc">
              {#if settingsStore.customRingtoneName !== null}
                {settingsStore.customRingtoneName}
                {#if settingsStore.customRingtoneDurationSec !== null && settingsStore.customRingtoneSize !== null}
                  ({Math.ceil(settingsStore.customRingtoneDurationSec)}s · {formatFileSize(settingsStore.customRingtoneSize)})
                {/if}
              {:else}
                Default chime (MP3, WAV, OGG, M4A · Max 10 MB · Clip up to 30s)
              {/if}
            </span>
          </div>

          <input
            bind:this={fileInputEl}
            type="file"
            accept=".mp3,.wav,.ogg,.m4a,audio/mpeg,audio/wav,audio/ogg,audio/mp4"
            class="sr-only-input"
            onchange={handleFileSelected}
          />

          <div class="ringtone-actions">
            <button
              type="button"
              class="option-btn"
              disabled={settingsStore.isRingtoneLoading || settingsStore.isTrimSaving}
              onclick={handleChooseRingtone}
            >
              {settingsStore.isRingtoneLoading ? "Checking..." : "Choose audio"}
            </button>
            <button
              type="button"
              class="option-btn"
              onclick={() => settingsStore.togglePreviewRingtone()}
            >
              {settingsStore.isPreviewPlaying ? "Stop" : "Test"}
            </button>
            {#if settingsStore.customRingtoneName !== null}
              <button
                type="button"
                class="option-btn"
                onclick={() => settingsStore.resetCustomRingtone()}
              >
                Reset
              </button>
            {/if}
          </div>

          {#if settingsStore.pendingTrimFile !== null}
            <div class="trim-panel" role="region" aria-label="Trim audio clip">
              <div class="trim-header">
                <span class="trim-title">
                  Trim "{settingsStore.pendingTrimFile.name}" (Total {formatSec(settingsStore.pendingTrimDurationSec)})
                </span>
                <span class="trim-badge">
                  {(trimEndSec - trimStartSec).toFixed(1)}s / 30s
                </span>
              </div>

              <AudioWaveformTrimmer
                peaks={settingsStore.pendingTrimPeaks}
                totalDurationSec={settingsStore.pendingTrimDurationSec}
                startSec={trimStartSec}
                endSec={trimEndSec}
                onChange={handleTrimRangeChange}
              />

              <div class="trim-range-meta">
                <span class="trim-time">{formatSec(trimStartSec)}</span>
                <span class="trim-time">{formatSec(trimEndSec)}</span>
              </div>

              <div class="ringtone-actions">
                <button
                  type="button"
                  class="option-btn"
                  onclick={() => settingsStore.toggleTrimPreview(trimStartSec, trimEndSec)}
                >
                  {settingsStore.isTrimPreviewPlaying ? "Stop preview" : "Preview selection"}
                </button>
                <button
                  type="button"
                  class="option-btn option-btn-active"
                  disabled={settingsStore.isTrimSaving}
                  onclick={() => settingsStore.confirmTrimAndSave(trimStartSec, trimEndSec)}
                >
                  {settingsStore.isTrimSaving ? "Trimming..." : "Save clip"}
                </button>
                <button
                  type="button"
                  class="option-btn"
                  disabled={settingsStore.isTrimSaving}
                  onclick={() => settingsStore.cancelTrim()}
                >
                  Cancel
                </button>
              </div>
            </div>
          {/if}

          {#if settingsStore.ringtoneError !== null}
            <p class="ringtone-error" role="alert">{settingsStore.ringtoneError}</p>
          {/if}
        </div>

        <div class="setting-row setting-row-col setting-row-divider">
          <div class="setting-info">
            <span class="setting-label">Default timer duration</span>
            <span class="setting-desc">Initial duration used when creating a new timer</span>
          </div>
          <div class="duration-options" role="radiogroup" aria-label="Default timer duration">
            {#each DURATION_OPTIONS as opt (opt.ms)}
              <button
                type="button"
                role="radio"
                aria-checked={settingsStore.defaultTimerDurationMs === opt.ms}
                class="option-btn"
                class:option-btn-active={settingsStore.defaultTimerDurationMs === opt.ms}
                onclick={() => settingsStore.setDefaultTimerDurationMs(opt.ms)}
              >
                {opt.label}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </div>

    <div class="settings-section">
      <h2 class="section-heading">About</h2>
      <div class="settings-card">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Lowatch</span>
            <span class="setting-desc">Multi-stopwatch and countdown timer desktop app</span>
          </div>
          <span class="version-text">v0.1.0</span>
        </div>
      </div>
    </div>
  </div>
</section>

<style>
  .settings-page {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px 36px;
  }

  .settings-container {
    width: 100%;
    max-width: 640px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .page-title {
    margin: 0;
    font-size: 1.375rem;
    font-weight: 600;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-heading {
    margin: 0;
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }

  .settings-card {
    background-color: var(--bg-card);
    border: 1px solid var(--border-card);
    border-radius: 10px;
    overflow: hidden;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 14px 16px;
  }

  .setting-row-col {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .setting-row-divider {
    border-top: 1px solid var(--border-card);
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .setting-label {
    font-size: 0.9375rem;
    font-weight: 500;
    color: var(--text-primary);
  }

  .setting-desc {
    font-size: 0.8125rem;
    color: var(--text-muted);
  }

  .toggle-switch {
    position: relative;
    width: 40px;
    height: 22px;
    border-radius: 6px;
    border: 1px solid var(--border-control);
    background-color: var(--bg-app);
    padding: 0;
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 150ms ease,
      border-color 150ms ease;
  }

  .toggle-switch:hover {
    border-color: var(--border-strong);
  }

  .toggle-switch-on {
    background-color: var(--accent-bg);
    border-color: var(--accent-bg);
  }

  .toggle-switch-on:hover {
    background-color: var(--accent-bg-hover);
    border-color: var(--accent-bg-hover);
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 4px;
    background-color: var(--text-muted);
    transition:
      transform 150ms ease,
      background-color 150ms ease;
  }

  .toggle-switch-on .toggle-thumb {
    transform: translateX(18px);
    background-color: var(--accent-fg);
  }

  .duration-options {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .option-btn {
    background-color: var(--bg-app);
    color: var(--text-secondary);
    border: 1px solid var(--border-card);
    border-radius: 6px;
    padding: 6px 12px;
    font-family: inherit;
    font-size: 0.8125rem;
    font-weight: 500;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }

  .option-btn:hover {
    background-color: var(--bg-control);
    color: var(--text-strong);
    border-color: var(--border-strong);
  }

  .option-btn-active {
    background-color: var(--accent-bg);
    color: var(--accent-fg);
    border-color: var(--accent-bg);
    font-weight: 600;
  }

  .option-btn-active:hover {
    background-color: var(--accent-bg-hover);
    color: var(--accent-fg);
    border-color: var(--accent-bg-hover);
  }

  .version-text {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.8125rem;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .opacity-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .opacity-value {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-variant-numeric: tabular-nums;
    font-size: 0.8125rem;
    font-weight: 600;
    color: var(--text-primary);
    flex-shrink: 0;
  }

  .opacity-slider {
    width: 100%;
    accent-color: var(--accent-bg);
    cursor: pointer;
  }

  .sr-only-input {
    display: none;
  }

  .ringtone-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .ringtone-error {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--danger-fg);
  }

  .trim-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    background-color: var(--bg-surface);
    border: 1px solid var(--border-card);
    border-radius: 8px;
  }

  .trim-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .trim-title {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-primary);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .trim-badge {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-variant-numeric: tabular-nums;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .trim-range-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .trim-time {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-variant-numeric: tabular-nums;
    font-size: 0.75rem;
    color: var(--text-primary);
  }
</style>
