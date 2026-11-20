<script lang="ts">
  import AudioWaveformTrimmer from "./AudioWaveformTrimmer.svelte";
  import {
    MAX_RINGTONE_DURATION_SEC,
    formatFileSize,
  } from "../ringtoneStorage";
  import StopwatchDial from "./StopwatchDial.svelte";
  import { settingsStore, type DialStyle, type ThemeMode } from "../stores/settings.svelte";

  let fileInputEl = $state<HTMLInputElement | null>(null);
  let trimStartSec = $state(0);
  let trimEndSec = $state(MAX_RINGTONE_DURATION_SEC);

  const THEME_OPTIONS: { label: string; value: ThemeMode }[] = [
    { label: "Dark", value: "dark" },
    { label: "Light", value: "light" },
    { label: "System", value: "system" },
  ];

  const DIAL_STYLE_OPTIONS: { label: string; value: DialStyle }[] = [
    { label: "Classic", value: "classic" },
    { label: "Sunburst", value: "sunburst" },
    { label: "Comet", value: "comet" },
    { label: "Dots", value: "dots" },
    { label: "Gauge", value: "gauge" },
  ];

  // 23 min 37.2 s: shows a partially filled minute ring and hour ring in previews.
  const PREVIEW_ELAPSED_MS = 23 * 60_000 + 37_200;

  const DURATION_OPTIONS = [
    { label: "1 min", ms: 60_000 },
    { label: "5 min", ms: 5 * 60_000 },
    { label: "10 min", ms: 10 * 60_000 },
    { label: "15 min", ms: 15 * 60_000 },
    { label: "25 min", ms: 25 * 60_000 },
    { label: "1 hour", ms: 60 * 60_000 },
  ];

  const SNOOZE_OPTIONS = [
    { label: "5 min", minutes: 5 },
    { label: "10 min", minutes: 10 },
    { label: "15 min", minutes: 15 },
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

        <div class="setting-row setting-row-col setting-row-divider">
          <div class="setting-info">
            <span class="setting-label">Dial style</span>
            <span class="setting-desc">Ring design used on stopwatch and timer cards</span>
          </div>
          <div class="dial-style-grid" role="radiogroup" aria-label="Dial style">
            {#each DIAL_STYLE_OPTIONS as opt (opt.value)}
              <button
                type="button"
                role="radio"
                aria-checked={settingsStore.dialStyle === opt.value}
                class="dial-style-option"
                class:dial-style-option-active={settingsStore.dialStyle === opt.value}
                onclick={() => settingsStore.setDialStyle(opt.value)}
              >
                <StopwatchDial
                  status="running"
                  elapsedMs={PREVIEW_ELAPSED_MS}
                  size={76}
                  variant={opt.value}
                />
                <span>{opt.label}</span>
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

        <div class="setting-row setting-row-divider">
          <div class="setting-info">
            <span class="setting-label">Close to system tray</span>
            <span class="setting-desc">
              Keep Lowatch running in the tray when closed so alarms and timers stay active
            </span>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={settingsStore.minimizeToTray}
            aria-label="Close to system tray"
            class="toggle-switch"
            class:toggle-switch-on={settingsStore.minimizeToTray}
            onclick={() => settingsStore.setMinimizeToTray(!settingsStore.minimizeToTray)}
          >
            <span class="toggle-thumb"></span>
          </button>
        </div>
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
      <h2 class="section-heading">Alarms</h2>
      <div class="settings-card">
        <div class="setting-row">
          <div class="setting-info">
            <span class="setting-label">Alarm sound</span>
            <span class="setting-desc">
              Loop the selected ringtone until the alarm is snoozed or dismissed
            </span>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={settingsStore.alarmSoundEnabled}
            aria-label="Alarm sound"
            class="toggle-switch"
            class:toggle-switch-on={settingsStore.alarmSoundEnabled}
            onclick={() =>
              settingsStore.setAlarmSoundEnabled(!settingsStore.alarmSoundEnabled)}
          >
            <span class="toggle-thumb"></span>
          </button>
        </div>

        <div class="setting-row setting-row-col setting-row-divider">
          <div class="setting-info">
            <span class="setting-label">Snooze duration</span>
            <span class="setting-desc">How long an alarm pauses when snoozed</span>
          </div>
          <div class="duration-options" role="radiogroup" aria-label="Snooze duration">
            {#each SNOOZE_OPTIONS as opt (opt.minutes)}
              <button
                type="button"
                role="radio"
                aria-checked={settingsStore.alarmSnoozeMinutes === opt.minutes}
                class="option-btn"
                class:option-btn-active={settingsStore.alarmSnoozeMinutes === opt.minutes}
                onclick={() => settingsStore.setAlarmSnoozeMinutes(opt.minutes)}
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
            <span class="setting-desc">Stopwatch, countdown timer, and alarm desktop app</span>
          </div>
          <span class="version-text">v1.0.0</span>
        </div>
      </div>
    </div>
  </div>
</section>

