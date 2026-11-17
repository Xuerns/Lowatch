<script lang="ts">
  import { settingsStore, type ThemeMode } from "../stores/settings.svelte";

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
</style>
