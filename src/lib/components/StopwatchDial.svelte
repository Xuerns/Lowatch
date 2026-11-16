<script lang="ts">
  import type { Snippet } from "svelte";
  import type { StopwatchStatus } from "../api";

  interface Props {
    status: StopwatchStatus;
    elapsedMs: number;
    size?: number;
    children: Snippet;
  }

  let { status, elapsedMs, size = 200, children }: Props = $props();

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
  let minuteFraction = $derived((safeElapsed % MINUTE_MS) / MINUTE_MS);
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
          class:tick-lit={status !== "idle" && tick.index <= litTicks}
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

<style>
  .dial {
    position: relative;
    width: var(--dial-size);
    height: var(--dial-size);
    max-width: 100%;
    aspect-ratio: 1 / 1;
    flex-shrink: 0;
    align-self: center;
  }

  .dial-svg {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    overflow: visible;
  }

  .tick {
    stroke: #363636;
    stroke-width: 1.2;
    stroke-linecap: round;
    transition: stroke 150ms ease;
  }

  .tick-major {
    stroke: #4a4a4a;
    stroke-width: 2;
  }

  .dial[data-status="running"] .tick-lit {
    stroke: #d4d4d4;
  }

  .dial[data-status="paused"] .tick-lit {
    stroke: #666666;
  }

  .dial-track {
    fill: none;
    stroke: #303030;
    stroke-width: 3;
  }

  .dial-progress {
    fill: none;
    stroke: transparent;
    stroke-width: 3;
    stroke-linecap: round;
    transition: stroke 150ms ease;
  }

  .dial[data-status="running"] .dial-progress {
    stroke: #f5f5f5;
  }

  .dial[data-status="paused"] .dial-progress {
    stroke: #5e5e5e;
  }

  .dial-hand {
    fill: #4a4a4a;
    stroke: #2b2b2b;
    stroke-width: 2;
    transition: fill 150ms ease;
  }

  .dial[data-status="running"] .dial-hand {
    fill: #ffffff;
  }

  .dial[data-status="paused"] .dial-hand {
    fill: #7a7a7a;
  }

  .dial-content {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
  }
</style>
