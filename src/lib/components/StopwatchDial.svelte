<script lang="ts">
  import type { Snippet } from "svelte";
  import type { StopwatchStatus, TimerStatus } from "../api";

  interface Props {
    status: StopwatchStatus | TimerStatus;
    elapsedMs?: number;
    progress?: number;
    size?: number;
    children: Snippet;
  }

  let { status, elapsedMs = 0, progress, size = 200, children }: Props = $props();

  const MINUTE_MS = 60_000;
  const TICK_COUNT = 60;
  const CENTER = 100;
  const ARC_RADIUS = 80;

  interface Tick {
    index: number;
    x1: number;
    y1: number;
    x2: number;
    y2: number;
    major: boolean;
  }

  const ticks: Tick[] = Array.from({ length: TICK_COUNT }, (_, index) => {
    const major = index % 5 === 0;
    const angle = (index / TICK_COUNT) * Math.PI * 2 - Math.PI / 2;
    const outer = 96;
    const inner = major ? 88 : 91.5;
    return {
      index,
      x1: CENTER + Math.cos(angle) * inner,
      y1: CENTER + Math.sin(angle) * inner,
      x2: CENTER + Math.cos(angle) * outer,
      y2: CENTER + Math.sin(angle) * outer,
      major,
    };
  });

  let safeElapsed = $derived(Math.max(0, elapsedMs));
  let minuteFraction = $derived(
    progress !== undefined
      ? Math.max(0, Math.min(1, progress))
      : (safeElapsed % MINUTE_MS) / MINUTE_MS
  );
  let litTicks = $derived(Math.floor(minuteFraction * TICK_COUNT));
  let progressOffset = $derived(100 - minuteFraction * 100);
  let handAngle = $derived(minuteFraction * 360);
</script>

<div class="dial" data-status={status} style:--dial-size="{size}px">
  <svg class="dial-svg" viewBox="0 0 200 200" aria-hidden="true">
    <g class="dial-ticks">
      {#each ticks as tick (tick.index)}
        <line
          class="tick"
          class:tick-major={tick.major}
          class:tick-lit={status !== "idle" &&
            status !== "completed" &&
            (progress === undefined || minuteFraction > 0) &&
            tick.index <= litTicks}
          x1={tick.x1}
          y1={tick.y1}
          x2={tick.x2}
          y2={tick.y2}
        />
      {/each}
    </g>

    <circle class="dial-track" cx={CENTER} cy={CENTER} r={ARC_RADIUS} />
    <circle
      class="dial-progress"
      cx={CENTER}
      cy={CENTER}
      r={ARC_RADIUS}
      pathLength="100"
      stroke-dasharray="100 100"
      stroke-dashoffset={progressOffset}
      transform="rotate(-90 {CENTER} {CENTER})"
    />

    <g transform="rotate({handAngle} {CENTER} {CENTER})">
      <circle class="dial-hand" cx={CENTER} cy={CENTER - ARC_RADIUS} r="4.5" />
    </g>
  </svg>

  <div class="dial-content">
    {@render children()}
  </div>
</div>

