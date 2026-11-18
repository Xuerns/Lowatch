<script lang="ts">
  import type { Snippet } from "svelte";
  import type { StopwatchStatus, TimerStatus } from "../api";
  import { settingsStore, type DialStyle } from "../stores/settings.svelte";

  interface Props {
    status: StopwatchStatus | TimerStatus;
    elapsedMs?: number;
    progress?: number;
    size?: number;
    variant?: DialStyle;
    children?: Snippet;
  }

  let { status, elapsedMs = 0, progress, size = 200, variant, children }: Props = $props();

  const MINUTE_MS = 60_000;
  const HOUR_MS = 60 * MINUTE_MS;
  const TICK_COUNT = 60;
  const CENTER = 100;
  const ARC_RADIUS = 80;

  interface Point {
    x: number;
    y: number;
  }

  /** Converts a clockwise angle in degrees, measured from 12 o'clock, to SVG coordinates. */
  function polar(radius: number, degrees: number): Point {
    const rad = ((degrees - 90) * Math.PI) / 180;
    return { x: CENTER + Math.cos(rad) * radius, y: CENTER + Math.sin(rad) * radius };
  }

  interface Tick {
    index: number;
    x1: number;
    y1: number;
    x2: number;
    y2: number;
    major: boolean;
  }

  function radialLine(index: number, deg: number, inner: number, outer: number, major: boolean): Tick {
    const a = polar(inner, deg);
    const b = polar(outer, deg);
    return { index, x1: a.x, y1: a.y, x2: b.x, y2: b.y, major };
  }

  // Classic ---------------------------------------------------------------
  const ticks: Tick[] = Array.from({ length: TICK_COUNT }, (_, index) => {
    const major = index % 5 === 0;
    return radialLine(index, (index / TICK_COUNT) * 360, major ? 88 : 91.5, 96, major);
  });

  // Dots ------------------------------------------------------------------
  const DOT_OUTER_RADIUS = 92;
  const DOT_INNER_RADIUS = 79;
  const HOUR_DOT_COUNT = 12;
  const outerDots: (Point & { index: number; major: boolean })[] = Array.from(
    { length: TICK_COUNT },
    (_, index) => ({ index, major: index % 5 === 0, ...polar(DOT_OUTER_RADIUS, (index / TICK_COUNT) * 360) })
  );
  const innerDots: (Point & { index: number })[] = Array.from({ length: HOUR_DOT_COUNT }, (_, index) => ({
    index,
    ...polar(DOT_INNER_RADIUS, (index / HOUR_DOT_COUNT) * 360),
  }));

  // Gauge -----------------------------------------------------------------
  const GAUGE_RADIUS = 82;
  const GAUGE_START = -135;
  const GAUGE_SWEEP = 270;
  const GAUGE_TICK_COUNT = 28;
  const gaugeStart = polar(GAUGE_RADIUS, GAUGE_START);
  const gaugeEnd = polar(GAUGE_RADIUS, GAUGE_START + GAUGE_SWEEP);
  const gaugePath = `M ${gaugeStart.x} ${gaugeStart.y} A ${GAUGE_RADIUS} ${GAUGE_RADIUS} 0 1 1 ${gaugeEnd.x} ${gaugeEnd.y}`;
  const gaugeTicks: Tick[] = Array.from({ length: GAUGE_TICK_COUNT }, (_, index) => {
    const major = index % 9 === 0;
    const deg = GAUGE_START + (index / (GAUGE_TICK_COUNT - 1)) * GAUGE_SWEEP;
    return radialLine(index, deg, major ? 89 : 91, 96, major);
  });

  // Sunburst --------------------------------------------------------------
  const BAR_COUNT = 72;
  const BAR_INNER = 82;
  const BAR_BASE = 4;
  const BAR_LIT = 6;
  const BAR_BOOST = 11;
  const BAR_FALLOFF = 7;

  // Comet -----------------------------------------------------------------
  const COMET_SEGMENTS = 32;
  const COMET_MAX_TAIL_DEG = 110;
  const COMET_HEAD_RADIUS = 4.4;

  // Shared state ----------------------------------------------------------
  let resolvedVariant = $derived<DialStyle>(variant ?? settingsStore.dialStyle);
  let safeElapsed = $derived(Math.max(0, elapsedMs));
  let minuteFraction = $derived(
    progress !== undefined
      ? Math.max(0, Math.min(1, progress))
      : (safeElapsed % MINUTE_MS) / MINUTE_MS
  );
  let active = $derived(
    status !== "idle" && status !== "completed" && (progress === undefined || minuteFraction > 0)
  );
  /** Whether the progress may wrap past 12 o'clock (stopwatch beyond its first minute). */
  let canWrap = $derived(progress === undefined && safeElapsed >= MINUTE_MS);
  let litTicks = $derived(Math.floor(minuteFraction * TICK_COUNT));
  let progressOffset = $derived(100 - minuteFraction * 100);
  let handAngle = $derived(minuteFraction * 360);

  let hourLit = $derived(Math.floor(((safeElapsed % HOUR_MS) / HOUR_MS) * HOUR_DOT_COUNT));
  let gaugeLit = $derived(Math.floor(minuteFraction * (GAUGE_TICK_COUNT - 1)));
  let gaugeHead = $derived(radialLine(0, GAUGE_START + minuteFraction * GAUGE_SWEEP, 73, 91, true));

  let bars = $derived.by(() => {
    const head = minuteFraction * BAR_COUNT;
    return Array.from({ length: BAR_COUNT }, (_, index) => {
      const lit = active && index <= head;
      // Distance behind the head; wraps across 12 o'clock so the wave stays continuous.
      let behind = head - index;
      if (behind < 0 && canWrap) behind += BAR_COUNT;
      const boost = active && behind >= 0 ? BAR_BOOST * Math.exp(-behind / BAR_FALLOFF) : 0;
      const length = (lit ? BAR_LIT : BAR_BASE) + boost;
      const bar = radialLine(index, (index / BAR_COUNT) * 360, BAR_INNER, BAR_INNER + length, false);
      return { ...bar, lit: lit || boost > 1 };
    });
  });

  let cometTail = $derived.by(() => {
    const tailDeg = active ? (canWrap ? COMET_MAX_TAIL_DEG : Math.min(COMET_MAX_TAIL_DEG, handAngle)) : 0;
    if (tailDeg <= 0) return [];
    const step = tailDeg / COMET_SEGMENTS;
    return Array.from({ length: COMET_SEGMENTS }, (_, index) => {
      const t = index / COMET_SEGMENTS;
      const from = polar(ARC_RADIUS, handAngle - step * index);
      // Slight overlap avoids hairline gaps between butt-capped segments.
      const to = polar(ARC_RADIUS, handAngle - step * (index + 1) - 0.4);
      return {
        index,
        d: `M ${from.x} ${from.y} A ${ARC_RADIUS} ${ARC_RADIUS} 0 0 0 ${to.x} ${to.y}`,
        width: COMET_HEAD_RADIUS * 1.6 * (1 - t) + 0.6,
        opacity: Math.pow(1 - t, 1.4),
      };
    });
  });
  let cometHead = $derived(polar(ARC_RADIUS, handAngle));
</script>

<div
  class="dial"
  data-status={status}
  data-variant={resolvedVariant}
  style:--dial-size="{size}px"
>
  <svg class="dial-svg" viewBox="0 0 200 200" aria-hidden="true">
    {#if resolvedVariant === "sunburst"}
      <g>
        {#each bars as bar (bar.index)}
          <line
            class="sun-bar"
            class:sun-bar-lit={bar.lit}
            x1={bar.x1}
            y1={bar.y1}
            x2={bar.x2}
            y2={bar.y2}
          />
        {/each}
      </g>
      <circle class="sun-core" cx={CENTER} cy={CENTER} r="76" />
    {:else if resolvedVariant === "comet"}
      <circle class="comet-track" cx={CENTER} cy={CENTER} r={ARC_RADIUS} />
      {#each [0, 90, 180, 270] as deg (deg)}
        {@const tick = radialLine(deg, deg, 88, 94, true)}
        <line class="comet-tick" x1={tick.x1} y1={tick.y1} x2={tick.x2} y2={tick.y2} />
      {/each}
      <g>
        {#each cometTail as seg (seg.index)}
          <path class="comet-tail" d={seg.d} stroke-width={seg.width} stroke-opacity={seg.opacity} />
        {/each}
      </g>
      <circle class="comet-dot" cx={cometHead.x} cy={cometHead.y} r={COMET_HEAD_RADIUS} />
    {:else if resolvedVariant === "dots"}
      <g>
        {#each outerDots as dot (dot.index)}
          <circle
            class="dot"
            class:dot-major={dot.major}
            class:dot-lit={active && dot.index <= litTicks}
            class:dot-head={active && dot.index === litTicks}
            cx={dot.x}
            cy={dot.y}
            r={active && dot.index === litTicks ? 3.4 : dot.major ? 2.3 : 1.4}
          />
        {/each}
      </g>
      {#if progress === undefined}
        <g>
          {#each innerDots as dot (dot.index)}
            <circle
              class="dot dot-hour"
              class:dot-lit={status !== "idle" && dot.index < hourLit}
              cx={dot.x}
              cy={dot.y}
              r="2.6"
            />
          {/each}
        </g>
      {/if}
    {:else if resolvedVariant === "gauge"}
      <g>
        {#each gaugeTicks as tick (tick.index)}
          <line
            class="tick"
            class:tick-major={tick.major}
            class:tick-lit={active && tick.index <= gaugeLit}
            x1={tick.x1}
            y1={tick.y1}
            x2={tick.x2}
            y2={tick.y2}
          />
        {/each}
      </g>
      <path class="gauge-track" d={gaugePath} />
      <path
        class="gauge-progress"
        d={gaugePath}
        pathLength="100"
        stroke-dasharray="100 100"
        stroke-dashoffset={progressOffset}
      />
      <line
        class="gauge-needle"
        x1={gaugeHead.x1}
        y1={gaugeHead.y1}
        x2={gaugeHead.x2}
        y2={gaugeHead.y2}
      />
    {:else}
      <g class="dial-ticks">
        {#each ticks as tick (tick.index)}
          <line
            class="tick"
            class:tick-major={tick.major}
            class:tick-lit={active && tick.index <= litTicks}
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
    {/if}
  </svg>

  <div class="dial-content">
    {@render children?.()}
  </div>
</div>
