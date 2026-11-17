<script lang="ts">
  import type { StopwatchDto } from "../api";
  import { settingsStore } from "../stores/settings.svelte";
  import { computeElapsedMs, formatElapsed } from "../time";
  import StopwatchDial from "./StopwatchDial.svelte";

  interface Props {
    stopwatch: StopwatchDto;
    nowMs: number;
    otherRunningCount?: number;
    onStart: (id: number) => Promise<void> | void;
    onPause: (id: number) => Promise<void> | void;
    onReset: (id: number) => Promise<void> | void;
    onFocus?: (id: number) => void;
    onExit: () => void;
  }

  let {
    stopwatch,
    nowMs,
    otherRunningCount = 0,
    onStart,
    onPause,
    onReset,
    onFocus,
    onExit,
  }: Props = $props();

  let confirmReset = $state(false);

  let elapsedMs = $derived(computeElapsedMs(stopwatch, nowMs));
  let formattedTime = $derived(formatElapsed(elapsedMs, settingsStore.showStopwatchTenths));
  let isResetDisabled = $derived(elapsedMs === 0 && stopwatch.status === "idle");

  let primaryActionLabel = $derived(
    stopwatch.status === "running"
      ? "Pause"
      : stopwatch.status === "paused"
        ? "Resume"
        : "Start"
  );

  async function handlePrimaryAction(): Promise<void> {
    confirmReset = false;
    if (stopwatch.status === "running") {
      await onPause(stopwatch.id);
    } else {
      await onStart(stopwatch.id);
    }
  }

  async function handleConfirmReset(): Promise<void> {
    confirmReset = false;
    await onReset(stopwatch.id);
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      if (confirmReset) {
        confirmReset = false;
      } else {
        onExit();
      }
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<section class="maximized-view" aria-label="Maximized stopwatch {stopwatch.name}">
  <header class="maximized-header">
    <div class="maximized-title-group">
      <h1 class="maximized-name" title={stopwatch.name}>{stopwatch.name}</h1>
      {#if otherRunningCount > 0}
        <span class="maximized-other-running" aria-label="{otherRunningCount} other running">
          +{otherRunningCount} running
        </span>
      {/if}
    </div>

    <div class="maximized-header-actions">
      {#if onFocus}
        <button
          type="button"
          class="btn-maximized-header"
          onclick={() => onFocus(stopwatch.id)}
          aria-label="Focus mode for {stopwatch.name}"
          title="Focus mode"
        >
          <svg
            width="15"
            height="15"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <rect x="3" y="3" width="18" height="18" rx="2" />
            <rect x="12" y="4" width="8" height="6" rx="1" fill="currentColor" stroke="none" />
          </svg>
        </button>
      {/if}

      <button
        type="button"
        class="btn-maximized-header"
        onclick={onExit}
        aria-label="Exit full view (Esc)"
        title="Exit full view (Esc)"
      >
        <svg
          width="15"
          height="15"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <polyline points="4 14 10 14 10 20" />
          <polyline points="20 10 14 10 14 4" />
          <line x1="14" y1="10" x2="21" y2="3" />
          <line x1="3" y1="21" x2="10" y2="14" />
        </svg>
      </button>
    </div>
  </header>

  <div class="maximized-stage">
    <StopwatchDial status={stopwatch.status} {elapsedMs} size={320}>
      <div class="maximized-time" data-status={stopwatch.status}>
        <span class="time-main"
          >{formattedTime.hours}:{formattedTime.minutes}:{formattedTime.seconds}</span
        >{#if settingsStore.showStopwatchTenths}<span class="maximized-tenths"
            >.{formattedTime.tenths}</span
          >{/if}
      </div>
    </StopwatchDial>

    {#if confirmReset}
      <div class="confirm-bar maximized-confirm" role="region" aria-label="Confirm reset stopwatch">
        <span class="confirm-text">Reset time to 00:00:00?</span>
        <div class="confirm-actions">
          <button type="button" class="btn-confirm-danger" onclick={handleConfirmReset}>
            Yes, Reset
          </button>
          <button
            type="button"
            class="btn-confirm-cancel"
            onclick={() => (confirmReset = false)}
          >
            Cancel
          </button>
        </div>
      </div>
    {:else}
      <div class="maximized-controls">
        <button
          type="button"
          class="btn-circle-primary btn-maximized-primary"
          data-status={stopwatch.status}
          onclick={handlePrimaryAction}
          aria-label="{primaryActionLabel} stopwatch {stopwatch.name}"
          title={primaryActionLabel}
        >
          {#if stopwatch.status === "running"}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <rect x="6" y="4" width="4" height="16" rx="1" />
              <rect x="14" y="4" width="4" height="16" rx="1" />
            </svg>
          {:else}
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <polygon points="7 4 20 12 7 20 7 4" />
            </svg>
          {/if}
        </button>

        <button
          type="button"
          class="btn-circle-secondary btn-maximized-secondary"
          disabled={isResetDisabled}
          onclick={() => (confirmReset = true)}
          aria-label="Reset stopwatch {stopwatch.name}"
          title="Reset"
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
            <path d="M3 3v5h5" />
          </svg>
        </button>
      </div>
    {/if}
  </div>
</section>
