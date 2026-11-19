<script lang="ts">
  import { alarmStore } from "../stores/alarms.svelte";
  import AlarmCard from "./AlarmCard.svelte";

  interface Props {
    editMode?: boolean;
  }

  let { editMode = false }: Props = $props();
</script>

{#if alarmStore.loading && alarmStore.items.length === 0}
  <div class="empty-state">
    <p class="empty-subtext">Loading alarms...</p>
  </div>
{:else if alarmStore.items.length === 0}
  <div class="empty-state">
    <svg
      class="empty-icon"
      width="36"
      height="36"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.6"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9" />
      <path d="M13.73 21a2 2 0 0 1-3.46 0" />
    </svg>
    <p class="empty-title">You don't have any alarms.</p>
    <p class="empty-subtext">Select "+" below to add a new alarm.</p>
  </div>
{:else}
  <section class="alarms-grid" aria-label="Alarm list">
    {#each alarmStore.items as alarm (alarm.id)}
      <AlarmCard
        {alarm}
        nowMs={alarmStore.nowMs}
        {editMode}
        onToggle={(id, enabled) => alarmStore.toggle(id, enabled)}
        onUpdate={(id, name, hour, minute, repeatDays) =>
          alarmStore.update(id, name, hour, minute, repeatDays)}
        onDelete={(id) => alarmStore.remove(id)}
      />
    {/each}
  </section>
{/if}
