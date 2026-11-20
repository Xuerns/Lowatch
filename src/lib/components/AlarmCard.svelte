<script lang="ts">
  import { tick } from "svelte";
  import type { AlarmDto } from "../api";
  import {
    REPEAT_DAY_OPTIONS,
    formatAlarmCountdown,
    formatAlarmTime,
    formatRepeatDays,
  } from "../alarm";

  interface Props {
    alarm: AlarmDto;
    nowMs: number;
    editMode?: boolean;
    onToggle: (id: number, enabled: boolean) => Promise<void> | void;
    onUpdate: (
      id: number,
      name: string,
      hour: number,
      minute: number,
      repeatDays: number
    ) => Promise<boolean> | boolean;
    onDelete: (id: number) => Promise<void> | void;
  }

  let { alarm, nowMs, editMode = false, onToggle, onUpdate, onDelete }: Props = $props();

  let isEditing = $state(false);
  let draftName = $state("");
  let draftTime = $state("07:00");
  let draftRepeatDays = $state(0);
  let nameTouched = $state(false);
  let timeTouched = $state(false);
  let submitAttempted = $state(false);
  let isSaving = $state(false);
  let confirmMode = $state<"none" | "delete">("none");
  let isDayUpdating = $state(false);
  let nameInputEl = $state<HTMLInputElement | null>(null);
  let timeInputEl = $state<HTMLInputElement | null>(null);

  let parsedTime = $derived.by(() => {
    const match = /^([01]\d|2[0-3]):([0-5]\d)$/.exec(draftTime);
    if (!match) return null;
    return { hour: Number(match[1]), minute: Number(match[2]) };
  });

  let trimmedName = $derived(draftName.trim());
  let nameInvalid = $derived((submitAttempted || nameTouched) && trimmedName.length === 0);
  let timeInvalid = $derived((submitAttempted || timeTouched) && parsedTime === null);
  let activeError = $derived(
    timeInvalid
      ? "Enter a valid time"
      : nameInvalid
        ? "Name cannot be empty"
        : null
  );

  let timeInputId = $derived(`alarm-time-${alarm.id}`);
  let nameInputId = $derived(`alarm-name-${alarm.id}`);
  let errorId = $derived(`alarm-error-${alarm.id}`);

  $effect(() => {
    if (!editMode) {
      isEditing = false;
      if (confirmMode === "delete") {
        confirmMode = "none";
      }
    }
  });

  async function beginEditing(): Promise<void> {
    confirmMode = "none";
    draftName = alarm.name;
    draftTime = formatAlarmTime(alarm.hour, alarm.minute);
    draftRepeatDays = alarm.repeat_days;
    nameTouched = false;
    timeTouched = false;
    submitAttempted = false;
    isEditing = true;
    await tick();
    timeInputEl?.focus();
  }

  function cancelEditing(): void {
    isEditing = false;
    confirmMode = "none";
    submitAttempted = false;
  }

  function toggleDraftDay(bit: number): void {
    draftRepeatDays =
      (draftRepeatDays & bit) !== 0 ? draftRepeatDays & ~bit : draftRepeatDays | bit;
  }

  async function handleSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!isEditing || isSaving) return;

    submitAttempted = true;
    if (parsedTime === null) {
      timeInputEl?.focus();
      return;
    }
    if (trimmedName.length === 0) {
      nameInputEl?.focus();
      return;
    }

    isSaving = true;
    try {
      const ok = await onUpdate(
        alarm.id,
        trimmedName,
        parsedTime.hour,
        parsedTime.minute,
        draftRepeatDays
      );
      if (ok) {
        isEditing = false;
      }
    } finally {
      isSaving = false;
    }
  }

  async function toggleDay(bit: number): Promise<void> {
    if (!editMode || isDayUpdating) return;
    const nextMask =
      (alarm.repeat_days & bit) !== 0 ? alarm.repeat_days & ~bit : alarm.repeat_days | bit;
    isDayUpdating = true;
    try {
      await onUpdate(alarm.id, alarm.name, alarm.hour, alarm.minute, nextMask);
    } finally {
      isDayUpdating = false;
    }
  }

  function handleEditorKeydown(event: KeyboardEvent): void {
    if (isEditing && event.key === "Escape") {
      event.preventDefault();
      cancelEditing();
    }
  }

  async function handleToggle(): Promise<void> {
    await onToggle(alarm.id, !alarm.enabled);
  }

  function requestDelete(): void {
    isEditing = false;
    confirmMode = "delete";
  }

  async function confirmDelete(): Promise<void> {
    confirmMode = "none";
    await onDelete(alarm.id);
  }

  function cancelConfirm(): void {
    confirmMode = "none";
  }
</script>

<svelte:window onkeydown={handleEditorKeydown} />

<article class="alarm-card" class:alarm-card-off={!alarm.enabled && !isEditing}>
  {#if isEditing}
    <form
      class="alarm-editor"
      onsubmit={handleSubmit}
      novalidate
      aria-label="Edit alarm {alarm.name}"
    >
      <div class="alarm-field">
        <label class="alarm-field-label" for={timeInputId}>Time</label>
        <input
          bind:this={timeInputEl}
          id={timeInputId}
          name="time"
          type="time"
          class="alarm-time-input"
          required
          aria-invalid={timeInvalid ? "true" : undefined}
          aria-describedby={timeInvalid ? errorId : undefined}
          bind:value={draftTime}
          onblur={() => {
            timeTouched = true;
          }}
        />
      </div>

      <div class="alarm-field">
        <label class="alarm-field-label" for={nameInputId}>Name</label>
        <input
          bind:this={nameInputEl}
          id={nameInputId}
          name="name"
          type="text"
          class="alarm-text-input"
          maxlength="50"
          required
          autocomplete="off"
          enterkeyhint="done"
          aria-invalid={nameInvalid ? "true" : undefined}
          aria-describedby={nameInvalid ? errorId : undefined}
          bind:value={draftName}
          onblur={() => {
            nameTouched = true;
          }}
        />
      </div>

      <div class="alarm-field">
        <span class="alarm-field-label">Repeat</span>
        <div class="alarm-days" role="group" aria-label="Repeat days">
          {#each REPEAT_DAY_OPTIONS as day (day.bit)}
            <button
              type="button"
              class="alarm-day"
              class:alarm-day-on={(draftRepeatDays & day.bit) !== 0}
              aria-pressed={(draftRepeatDays & day.bit) !== 0}
              aria-label={day.label}
              title={day.label}
              onclick={() => toggleDraftDay(day.bit)}
            >
              {day.short}
            </button>
          {/each}
        </div>
      </div>

      {#if activeError !== null}
        <p id={errorId} class="alarm-editor-error" role="alert" aria-live="polite">
          {activeError}
        </p>
      {/if}

      <div class="alarm-editor-actions">
        <button type="button" class="alarm-cancel-btn" onclick={cancelEditing}>
          Cancel
        </button>
        <button type="submit" class="alarm-save-btn" disabled={isSaving}>
          Save
        </button>
      </div>
    </form>
  {:else}
    <div class="alarm-card-top">
      <span class="alarm-time">
        {formatAlarmTime(alarm.hour, alarm.minute)}
      </span>

      <div class="alarm-actions">
        {#if editMode}
          <button
            type="button"
            class="btn-icon-ghost"
            onclick={beginEditing}
            aria-label="Edit alarm {alarm.name}"
            title="Edit"
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
              <path d="M12 20h9" />
              <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4Z" />
            </svg>
          </button>
          <button
            type="button"
            class="btn-icon-ghost btn-delete-trigger"
            onclick={requestDelete}
            aria-label="Delete alarm {alarm.name}"
            title="Delete"
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
              <polyline points="3 6 5 6 21 6" />
              <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
              <path d="M10 11v6" />
              <path d="M14 11v6" />
              <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
            </svg>
          </button>
        {/if}

        <button
          type="button"
          role="switch"
          aria-checked={alarm.enabled}
          aria-label="Enable alarm {alarm.name}"
          class="toggle-switch"
          class:toggle-switch-on={alarm.enabled}
          onclick={handleToggle}
        >
          <span class="toggle-thumb"></span>
        </button>
      </div>
    </div>

    <div class="alarm-status-line">
      <svg
        class="alarm-bell"
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
        <path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9" />
        <path d="M13.73 21a2 2 0 0 1-3.46 0" />
      </svg>
      <span class="alarm-next">{formatAlarmCountdown(alarm, nowMs)}</span>
    </div>

    <h2
      class="alarm-name"
      ondblclick={() => {
        if (editMode) void beginEditing();
      }}
      title={editMode ? "Double-click to edit" : alarm.name}
    >
      {alarm.name}
    </h2>

    <div
      class="alarm-days"
      role="group"
      aria-label={editMode
        ? "Repeat days"
        : `Repeats: ${formatRepeatDays(alarm.repeat_days)}`}
    >
      {#each REPEAT_DAY_OPTIONS as day (day.bit)}
        {#if editMode}
          <button
            type="button"
            class="alarm-day"
            class:alarm-day-on={(alarm.repeat_days & day.bit) !== 0}
            aria-pressed={(alarm.repeat_days & day.bit) !== 0}
            aria-label={day.label}
            title={day.label}
            disabled={isDayUpdating}
            onclick={() => toggleDay(day.bit)}
          >
            {day.short}
          </button>
        {:else}
          <span
            class="alarm-day"
            class:alarm-day-on={(alarm.repeat_days & day.bit) !== 0}
            aria-hidden="true"
            title={day.label}
          >
            {day.short}
          </span>
        {/if}
      {/each}
    </div>

    {#if confirmMode === "delete"}
      <div class="confirm-bar" role="region" aria-label="Confirm delete alarm">
        <span class="confirm-text">Delete "{alarm.name}"?</span>
        <div class="confirm-actions">
          <button type="button" class="btn-confirm-danger" onclick={confirmDelete}>
            Yes, Delete
          </button>
          <button type="button" class="btn-confirm-cancel" onclick={cancelConfirm}>
            Cancel
          </button>
        </div>
      </div>
    {/if}
  {/if}
</article>
