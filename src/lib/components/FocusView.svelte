<script lang="ts">
  import type { StopwatchDto } from "../api";
  import { computeElapsedMs, formatElapsed } from "../time";
  import StopwatchDial from "./StopwatchDial.svelte";

  interface Props {
    stopwatch: StopwatchDto;
    nowMs: number;
    onStart: (id: number) => Promise<void> | void;
    onPause: (id: number) => Promise<void> | void;
    onExit: () => Promise<void> | void;
  }

  let { stopwatch, nowMs, onStart, onPause, onExit }: Props = $props();

  let elapsedMs = $derived(computeElapsedMs(stopwatch, nowMs));
  let formattedTime = $derived(formatElapsed(elapsedMs, false));

  let primaryActionLabel = $derived(
    stopwatch.status === "running"
      ? "Pause"
      : stopwatch.status === "paused"
        ? "Resume"
        : "Start"
  );

  async function handlePrimaryAction(): Promise<void> {
    if (stopwatch.status === "running") {
      await onPause(stopwatch.id);
    } else {
      await onStart(stopwatch.id);
    }
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      void onExit();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<section
  class="focus-widget"
  data-tauri-drag-region
  aria-label="Focus mode stopwatch {stopwatch.name}"
>
  <div class="focus-top" data-tauri-drag-region>
    <span class="focus-name" data-tauri-drag-region title={stopwatch.name}>
      {stopwatch.name}
    </span>
    <button
      type="button"
      class="btn-exit"
      onclick={() => onExit()}
      aria-label="Return to full view (Esc)"
      title="Return to full view (Esc)"
    >
      <svg
        width="14"
        height="14"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M15 3h6v6" />
        <path d="M9 21H3v-6" />
        <path d="M21 3l-7 7" />
        <path d="M3 21l7-7" />
      </svg>
    </button>
  </div>

  <div class="focus-body" data-tauri-drag-region>
    <StopwatchDial status={stopwatch.status} {elapsedMs} size={188}>
      <div
        class="focus-time"
        data-status={stopwatch.status}
        data-tauri-drag-region
      >
        {formattedTime.formatted}
      </div>
      <button
        type="button"
        class="btn-action"
        data-status={stopwatch.status}
        onclick={handlePrimaryAction}
        aria-label="{primaryActionLabel} stopwatch {stopwatch.name}"
        title={primaryActionLabel}
      >
        {#if stopwatch.status === "running"}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <rect x="6" y="4" width="4" height="16" rx="1" />
            <rect x="14" y="4" width="4" height="16" rx="1" />
          </svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <polygon points="7 4 20 12 7 20 7 4" />
          </svg>
        {/if}
      </button>
    </StopwatchDial>
  </div>
</section>

