<script lang="ts">
  interface Props {
    peaks: number[];
    totalDurationSec: number;
    startSec: number;
    endSec: number;
    minDurationSec?: number;
    maxDurationSec?: number;
    onChange: (startSec: number, endSec: number) => void;
  }

  let {
    peaks,
    totalDurationSec,
    startSec,
    endSec,
    minDurationSec = 1,
    maxDurationSec = 30,
    onChange,
  }: Props = $props();

  let trackEl = $state<HTMLDivElement | null>(null);
  let dragMode = $state<"none" | "move" | "left" | "right">("none");
  let dragOriginX = 0;
  let dragInitialStart = 0;
  let dragInitialEnd = 0;

  let startPct = $derived(
    totalDurationSec > 0 ? Math.max(0, Math.min(100, (startSec / totalDurationSec) * 100)) : 0
  );
  let endPct = $derived(
    totalDurationSec > 0 ? Math.max(0, Math.min(100, (endSec / totalDurationSec) * 100)) : 100
  );
  let widthPct = $derived(Math.max(0, endPct - startPct));

  function beginDrag(
    event: PointerEvent,
    mode: "move" | "left" | "right"
  ): void {
    event.stopPropagation();
    event.preventDefault();
    dragMode = mode;
    dragOriginX = event.clientX;
    dragInitialStart = startSec;
    dragInitialEnd = endSec;
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  function handlePointerMove(event: PointerEvent): void {
    if (dragMode === "none" || !trackEl || totalDurationSec <= 0) return;
    const rect = trackEl.getBoundingClientRect();
    if (rect.width <= 0) return;

    const deltaSec = ((event.clientX - dragOriginX) / rect.width) * totalDurationSec;

    if (dragMode === "move") {
      const span = dragInitialEnd - dragInitialStart;
      const nextStart = Math.max(0, Math.min(totalDurationSec - span, dragInitialStart + deltaSec));
      onChange(roundHalf(nextStart), roundHalf(nextStart + span));
    } else if (dragMode === "left") {
      const minAllowed = Math.max(0, dragInitialEnd - maxDurationSec);
      const maxAllowed = dragInitialEnd - minDurationSec;
      const nextStart = Math.max(minAllowed, Math.min(maxAllowed, dragInitialStart + deltaSec));
      onChange(roundHalf(nextStart), dragInitialEnd);
    } else if (dragMode === "right") {
      const minAllowed = dragInitialStart + minDurationSec;
      const maxAllowed = Math.min(totalDurationSec, dragInitialStart + maxDurationSec);
      const nextEnd = Math.max(minAllowed, Math.min(maxAllowed, dragInitialEnd + deltaSec));
      onChange(dragInitialStart, roundHalf(nextEnd));
    }
  }

  function endDrag(event: PointerEvent): void {
    if (dragMode === "none") return;
    dragMode = "none";
    try {
      (event.currentTarget as HTMLElement).releasePointerCapture(event.pointerId);
    } catch {
      // Ignore if pointer capture already released
    }
  }

  function handleTrackClick(event: MouseEvent): void {
    if (dragMode !== "none" || !trackEl || totalDurationSec <= 0) return;
    const rect = trackEl.getBoundingClientRect();
    if (rect.width <= 0) return;
    const clickSec = ((event.clientX - rect.left) / rect.width) * totalDurationSec;
    const span = endSec - startSec;
    const nextStart = Math.max(0, Math.min(totalDurationSec - span, clickSec - span / 2));
    onChange(roundHalf(nextStart), roundHalf(nextStart + span));
  }

  function roundHalf(val: number): number {
    return Math.round(val * 10) / 10;
  }
</script>

<div
  bind:this={trackEl}
  class="waveform-track"
  role="slider"
  aria-label="Audio trim range"
  aria-valuemin={0}
  aria-valuemax={totalDurationSec}
  aria-valuenow={startSec}
  tabindex="0"
  onclick={handleTrackClick}
  onkeydown={(e) => {
    const step = 1;
    const span = endSec - startSec;
    if (e.key === "ArrowLeft") {
      e.preventDefault();
      const nextStart = Math.max(0, startSec - step);
      onChange(roundHalf(nextStart), roundHalf(nextStart + span));
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      const nextStart = Math.min(totalDurationSec - span, startSec + step);
      onChange(roundHalf(nextStart), roundHalf(nextStart + span));
    }
  }}
>
  <div class="waveform-bars" aria-hidden="true">
    {#each peaks as peak, idx (idx)}
      <span class="wave-bar" style:height="{Math.round(peak * 100)}%"></span>
    {/each}
  </div>

  <!-- Darkened regions outside the active frame -->
  <div class="mask-outside mask-left" style:width="{startPct}%"></div>
  <div
    class="mask-outside mask-right"
    style:left="{endPct}%"
    style:width="{Math.max(0, 100 - endPct)}%"
  ></div>

  <!-- Adjustable selection frame -->
  <div
    class="selection-frame"
    class:selection-dragging={dragMode === "move"}
    role="presentation"
    style:left="{startPct}%"
    style:width="{widthPct}%"
    onpointerdown={(e) => beginDrag(e, "move")}
    onpointermove={handlePointerMove}
    onpointerup={endDrag}
    onpointercancel={endDrag}
  >
    <div
      class="frame-handle handle-left"
      role="presentation"
      onpointerdown={(e) => beginDrag(e, "left")}
      onpointermove={handlePointerMove}
      onpointerup={endDrag}
      onpointercancel={endDrag}
    >
      <span class="handle-grip"></span>
    </div>

    <div
      class="frame-handle handle-right"
      role="presentation"
      onpointerdown={(e) => beginDrag(e, "right")}
      onpointermove={handlePointerMove}
      onpointerup={endDrag}
      onpointercancel={endDrag}
    >
      <span class="handle-grip"></span>
    </div>
  </div>
</div>

