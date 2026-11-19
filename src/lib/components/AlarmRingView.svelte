<script lang="ts">
  import type { AlarmDto } from "../api";
  import { ALARM_SNOOZE_MINUTES, formatAlarmTime, formatRepeatDays } from "../alarm";

  interface Props {
    alarm: AlarmDto;
    nowMs: number;
    onSnooze: (id: number) => Promise<void> | void;
    onDismiss: (id: number) => Promise<void> | void;
  }

  let { alarm, nowMs, onSnooze, onDismiss }: Props = $props();

  let pending = $state<"snooze" | "dismiss" | null>(null);
  let snoozeBtnEl = $state<HTMLButtonElement | null>(null);

  let clockLabel = $derived.by(() => {
    const now = new Date(nowMs);
    return `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}`;
  });

  $effect(() => {
    if (snoozeBtnEl) {
      snoozeBtnEl.focus();
    }
  });

  async function handleSnooze(): Promise<void> {
    if (pending !== null) return;
    pending = "snooze";
    try {
      await onSnooze(alarm.id);
    } finally {
      pending = null;
    }
  }

  async function handleDismiss(): Promise<void> {
    if (pending !== null) return;
    pending = "dismiss";
    try {
      await onDismiss(alarm.id);
    } finally {
      pending = null;
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.preventDefault();
      void handleDismiss();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="alarm-ring-overlay">
  <div
    class="alarm-ring-panel"
    role="alertdialog"
    aria-modal="true"
    aria-labelledby="alarm-ring-title"
    aria-describedby="alarm-ring-repeat"
  >
    <p class="alarm-ring-kicker">Alarm ringing</p>
    <p class="alarm-ring-time">{formatAlarmTime(alarm.hour, alarm.minute)}</p>
    <h1 class="alarm-ring-name" id="alarm-ring-title">{alarm.name}</h1>
    <p class="alarm-ring-repeat" id="alarm-ring-repeat">
      {formatRepeatDays(alarm.repeat_days)} · Local time {clockLabel}
    </p>

    <div class="alarm-ring-actions">
      <button
        bind:this={snoozeBtnEl}
        type="button"
        class="alarm-ring-btn alarm-ring-btn-primary"
        disabled={pending !== null}
        onclick={handleSnooze}
      >
        {pending === "snooze" ? "Snoozing..." : `Snooze ${ALARM_SNOOZE_MINUTES} min`}
      </button>
      <button
        type="button"
        class="alarm-ring-btn"
        disabled={pending !== null}
        onclick={handleDismiss}
      >
        {pending === "dismiss" ? "Dismissing..." : "Dismiss"}
      </button>
    </div>
  </div>
</div>
