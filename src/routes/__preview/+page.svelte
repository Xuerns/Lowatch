<script lang="ts">
  import type { AlarmDto } from "$lib/api";
  import AlarmsView from "$lib/components/AlarmsView.svelte";
  import { alarmStore } from "$lib/stores/alarms.svelte";

  const base = Date.now();
  const inAboutOneDay = new Date(base + (21 * 60 + 6) * 60_000);

  alarmStore.loading = false;
  alarmStore.nowMs = base;
  alarmStore.items = [
    {
      id: 1,
      name: "Good morning",
      hour: inAboutOneDay.getHours(),
      minute: inAboutOneDay.getMinutes(),
      repeat_days: 0b1111111,
      enabled: false,
      snoozed_until_ms: null,
      next_ring_at_ms: null,
      position: 0,
    },
    {
      id: 2,
      name: "Wake up",
      hour: inAboutOneDay.getHours(),
      minute: inAboutOneDay.getMinutes(),
      repeat_days: 0b0011111,
      enabled: true,
      snoozed_until_ms: null,
      next_ring_at_ms: base + (21 * 60 + 6) * 60_000,
      position: 1,
    },
    {
      id: 3,
      name: "Call with Nia",
      hour: new Date(base + 2 * 3600_000).getHours(),
      minute: new Date(base + 2 * 3600_000).getMinutes(),
      repeat_days: 0b0110000,
      enabled: true,
      snoozed_until_ms: null,
      next_ring_at_ms: base + 2 * 3600_000,
      position: 2,
    },
  ] satisfies AlarmDto[];
</script>

<div style="display: flex; height: 100vh; background-color: var(--bg-app);">
  <AlarmsView editMode={false} />
</div>
