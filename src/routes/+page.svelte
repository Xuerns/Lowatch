<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import AlarmRingView from "$lib/components/AlarmRingView.svelte";
  import AlarmsView from "$lib/components/AlarmsView.svelte";
  import FocusView from "$lib/components/FocusView.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import Sidebar, { type AppPage } from "$lib/components/Sidebar.svelte";
  import StopwatchCard from "$lib/components/StopwatchCard.svelte";
  import StopwatchMaximizedView from "$lib/components/StopwatchMaximizedView.svelte";
  import TimerCard from "$lib/components/TimerCard.svelte";
  import { alarmStore } from "$lib/stores/alarms.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { stopwatchStore } from "$lib/stores/stopwatches.svelte";
  import { timerStore } from "$lib/stores/timers.svelte";

  const appWindow = getCurrentWindow();

  let activePage = $state<AppPage>("stopwatch");
  let isSidebarCollapsed = $state(false);
  let isCreating = $state(false);
  let isMaximized = $state(false);
  let isEditMode = $state(false);
  let maximizedStopwatchId = $state<number | null>(null);

  let maximizedStopwatch = $derived(
    maximizedStopwatchId !== null
      ? (stopwatchStore.items.find((item) => item.id === maximizedStopwatchId) ?? null)
      : null
  );

  let otherRunningCount = $derived(
    maximizedStopwatch !== null
      ? stopwatchStore.items.filter(
          (item) => item.id !== maximizedStopwatch.id && item.status === "running"
        ).length
      : 0
  );

  let activeErrorMessage = $derived(
    activePage === "stopwatch"
      ? stopwatchStore.errorMessage
      : activePage === "timer"
        ? timerStore.errorMessage
        : activePage === "alarm"
          ? alarmStore.errorMessage
          : null
  );

  let activePageTitle = $derived(
    activePage === "stopwatch" ? "Stopwatch" : activePage === "timer" ? "Timer" : "Alarm"
  );

  let activePageNoun = $derived(
    activePage === "stopwatch" ? "stopwatch" : activePage === "timer" ? "timer" : "alarm"
  );

  let activePagePlural = $derived(
    activePage === "stopwatch" ? "stopwatches" : activePage === "timer" ? "timers" : "alarms"
  );

  onMount(() => {
    void stopwatchStore.load();
    void timerStore.load();
    void alarmStore.load();

    let unlistenResize: (() => void) | undefined;
    void (async () => {
      try {
        isMaximized = await appWindow.isMaximized();
        unlistenResize = await appWindow.onResized(async () => {
          isMaximized = await appWindow.isMaximized();
        });
      } catch {
        // Ignore outside Tauri runtime (e.g. browser preview)
      }
    })();

    return () => {
      unlistenResize?.();
      stopwatchStore.destroy();
      timerStore.destroy();
      alarmStore.destroy();
    };
  });

  $effect(() => {
    if (
      !stopwatchStore.loading &&
      stopwatchStore.focusedId !== null &&
      stopwatchStore.focusedStopwatch === null
    ) {
      void stopwatchStore.exitFocus();
    }
  });

  $effect(() => {
    if (
      !stopwatchStore.loading &&
      maximizedStopwatchId !== null &&
      maximizedStopwatch === null
    ) {
      maximizedStopwatchId = null;
    }
  });

  $effect(() => {
    if (alarmStore.ringingAlarm !== null && stopwatchStore.focusedId !== null) {
      void stopwatchStore.exitFocus();
    }
  });

  async function handleMinimize(): Promise<void> {
    await appWindow.minimize();
  }

  async function handleToggleMaximize(): Promise<void> {
    await appWindow.toggleMaximize();
    isMaximized = await appWindow.isMaximized();
  }

  async function handleClose(): Promise<void> {
    await appWindow.close();
  }

  async function handleQuickCreate(): Promise<void> {
    if (isCreating) return;
    isCreating = true;
    try {
      if (activePage === "stopwatch") {
        await stopwatchStore.create();
      } else if (activePage === "timer") {
        await timerStore.create(undefined, settingsStore.defaultTimerDurationMs);
      } else if (activePage === "alarm") {
        await alarmStore.create();
      }
    } finally {
      isCreating = false;
    }
  }

  function handleSelectPage(page: AppPage): void {
    activePage = page;
    isEditMode = false;
    maximizedStopwatchId = null;
  }

  function handleDismissError(): void {
    if (activePage === "stopwatch") {
      stopwatchStore.clearError();
    } else if (activePage === "timer") {
      timerStore.clearError();
    } else if (activePage === "alarm") {
      alarmStore.clearError();
    }
  }

  async function handleFocusStopwatch(id: number): Promise<void> {
    await stopwatchStore.enterFocus(id);
  }
</script>

{#if alarmStore.ringingAlarm !== null}
  <AlarmRingView
    alarm={alarmStore.ringingAlarm}
    nowMs={alarmStore.nowMs}
    snoozeMinutes={settingsStore.alarmSnoozeMinutes}
    onSnooze={(id) => alarmStore.snooze(id)}
    onDismiss={(id) => alarmStore.dismiss(id)}
  />
{/if}

{#if stopwatchStore.focusedStopwatch !== null}
  <FocusView
    stopwatch={stopwatchStore.focusedStopwatch}
    nowMs={stopwatchStore.nowMs}
    onStart={(id) => stopwatchStore.start(id)}
    onPause={(id) => stopwatchStore.pause(id)}
    onExit={() => stopwatchStore.exitFocus()}
  />
{:else}
  <main class="app-shell">
    <header class="titlebar" data-tauri-drag-region>
      <div class="titlebar-left" data-tauri-drag-region>
        {#if !isMaximized}
          <button
            type="button"
            class="titlebar-toggle"
            onclick={() => (isSidebarCollapsed = !isSidebarCollapsed)}
            aria-label={isSidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
            title={isSidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
          >
            <svg
              width="15"
              height="15"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <rect x="3" y="3" width="18" height="18" rx="2" />
              <line x1="9" y1="3" x2="9" y2="21" />
            </svg>
          </button>
        {/if}

        <div class="titlebar-brand" data-tauri-drag-region>
          <img src="/logo.png" alt="Lowatch logo" class="titlebar-logo" />
          <span class="titlebar-title" data-tauri-drag-region>Lowatch</span>
        </div>
      </div>

      <div class="titlebar-controls">
        <button
          type="button"
          class="win-btn"
          onclick={handleMinimize}
          aria-label="Minimize"
          title="Minimize"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <line x1="0" y1="5" x2="10" y2="5" stroke="currentColor" stroke-width="1" />
          </svg>
        </button>

        <button
          type="button"
          class="win-btn"
          onclick={handleToggleMaximize}
          aria-label={isMaximized ? "Restore" : "Maximize"}
          title={isMaximized ? "Restore" : "Maximize"}
        >
          {#if isMaximized}
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <rect x="0.5" y="2.5" width="7" height="7" fill="none" stroke="currentColor" stroke-width="1" />
              <polyline points="2.5,2.5 2.5,0.5 9.5,0.5 9.5,7.5 7.5,7.5" fill="none" stroke="currentColor" stroke-width="1" />
            </svg>
          {:else}
            <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
              <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1" />
            </svg>
          {/if}
        </button>

        <button
          type="button"
          class="win-btn win-btn-close"
          onclick={handleClose}
          aria-label="Close"
          title="Close"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" aria-hidden="true">
            <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" stroke-width="1.1" />
            <line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" stroke-width="1.1" />
          </svg>
        </button>
      </div>
    </header>

    <div class="workspace">
      <Sidebar
        {activePage}
        collapsed={!isMaximized && isSidebarCollapsed}
        stopwatchRunningCount={stopwatchStore.runningCount}
        timerRunningCount={timerStore.runningCount}
        alarmActiveCount={alarmStore.activeCount}
        onSelectPage={handleSelectPage}
      />

      <div class="content-area">
        {#if settingsStore.isNotificationPlaying}
          <div
            class="notification-banner"
            role="status"
            aria-live="polite"
            transition:fly={{ y: -16, duration: 200 }}
          >
            <svg
              class="notification-icon"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="1.9"
              stroke-linecap="round"
              stroke-linejoin="round"
              aria-hidden="true"
            >
              <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
              <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
              <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
            </svg>
            <div class="notification-copy">
              <span class="notification-title">
                {settingsStore.activeNotificationTimerName
                  ? `${settingsStore.activeNotificationTimerName} finished`
                  : "Timer finished"}
              </span>
              <span class="notification-sub">Ringtone playing</span>
            </div>
            <button
              type="button"
              class="btn-stop-sound"
              onclick={() => settingsStore.stopTimerNotification()}
            >
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="currentColor"
                aria-hidden="true"
              >
                <rect x="4" y="4" width="16" height="16" rx="2" />
              </svg>
              <span>Stop</span>
            </button>
          </div>
        {/if}

        {#if activeErrorMessage !== null}
          <div class="error-banner" role="alert">
            <span class="error-text">{activeErrorMessage}</span>
            <button
              type="button"
              class="btn-dismiss"
              onclick={handleDismissError}
            >
              Dismiss
            </button>
          </div>
        {/if}

        {#if activePage === "stopwatch"}
          {#if stopwatchStore.loading && stopwatchStore.items.length === 0}
            <div class="empty-state">
              <p class="empty-subtext">Loading stopwatches...</p>
            </div>
          {:else if stopwatchStore.items.length === 0}
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
                <circle cx="12" cy="13" r="8" />
                <path d="M12 9v4l2.5 1.5" />
                <path d="M10 2h4" />
                <path d="M12 2v3" />
              </svg>
              <p class="empty-title">You don't have any stopwatches.</p>
              <p class="empty-subtext">Select "+" below to add a new stopwatch.</p>
            </div>
          {:else if maximizedStopwatch !== null}
            <StopwatchMaximizedView
              stopwatch={maximizedStopwatch}
              nowMs={stopwatchStore.nowMs}
              {otherRunningCount}
              onStart={(id) => stopwatchStore.start(id)}
              onPause={(id) => stopwatchStore.pause(id)}
              onReset={(id) => stopwatchStore.reset(id)}
              onFocus={handleFocusStopwatch}
              onExit={() => (maximizedStopwatchId = null)}
            />
          {:else}
            <section class="stopwatch-grid" aria-label="Stopwatch list">
              {#each stopwatchStore.items as item (item.id)}
                <StopwatchCard
                  stopwatch={item}
                  nowMs={stopwatchStore.nowMs}
                  editMode={isEditMode}
                  onStart={(id) => stopwatchStore.start(id)}
                  onPause={(id) => stopwatchStore.pause(id)}
                  onReset={(id) => stopwatchStore.reset(id)}
                  onRename={(id, name) => stopwatchStore.rename(id, name)}
                  onDelete={(id) => stopwatchStore.remove(id)}
                  onFocus={handleFocusStopwatch}
                  onMaximize={(id) => (maximizedStopwatchId = id)}
                />
              {/each}
            </section>
          {/if}
        {:else if activePage === "alarm"}
          <AlarmsView editMode={isEditMode} />
        {:else if activePage === "settings"}
          <SettingsView />
        {:else if timerStore.loading && timerStore.items.length === 0}
          <div class="empty-state">
            <p class="empty-subtext">Loading timers...</p>
          </div>
        {:else if timerStore.items.length === 0}
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
              <path d="M5 2h14" />
              <path d="M5 22h14" />
              <path d="M7 2v4.5a5 5 0 0 0 1.46 3.54L12 13.5l3.54-3.46A5 5 0 0 0 17 6.5V2" />
              <path d="M7 22v-4.5a5 5 0 0 1 1.46-3.54L12 10.5l3.54 3.46A5 5 0 0 1 17 17.5V22" />
            </svg>
            <p class="empty-title">You don't have any timers.</p>
            <p class="empty-subtext">Select "+" below to add a new timer.</p>
          </div>
        {:else}
          <section class="stopwatch-grid" aria-label="Timer list">
            {#each timerStore.items as item (item.id)}
              <TimerCard
                timer={item}
                nowMs={timerStore.nowMs}
                editMode={isEditMode}
                onStart={(id) => timerStore.start(id)}
                onPause={(id) => timerStore.pause(id)}
                onReset={(id) => timerStore.reset(id)}
                onRename={(id, name) => timerStore.rename(id, name)}
                onSetDuration={(id, ms) => timerStore.setDuration(id, ms)}
                onDelete={(id) => timerStore.remove(id)}
              />
            {/each}
          </section>
        {/if}
      </div>
    </div>

    {#if activePage !== "settings" && !(activePage === "stopwatch" && maximizedStopwatch !== null)}
      <div
        class="fab-bar"
        role="toolbar"
        aria-label="{activePageTitle} actions"
      >
      <button
        type="button"
        class="fab-btn"
        class:fab-btn-active={isEditMode}
        onclick={() => (isEditMode = !isEditMode)}
        aria-pressed={isEditMode}
        aria-label={isEditMode ? "Done editing" : `Edit ${activePagePlural}`}
        title={isEditMode ? "Done editing" : `Edit ${activePagePlural}`}
      >
        {#if isEditMode}
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <polyline points="20 6 9 17 4 12" />
          </svg>
        {:else}
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.8"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <path d="M12 20h9" />
            <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4Z" />
          </svg>
        {/if}
      </button>

      <button
        type="button"
        class="fab-btn"
        disabled={isCreating}
        onclick={handleQuickCreate}
        aria-label={`New ${activePageNoun}`}
        title={`New ${activePageNoun}`}
      >
        <svg
          width="17"
          height="17"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
      </button>
    </div>
    {/if}
  </main>
{/if}

