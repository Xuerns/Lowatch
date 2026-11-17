<script lang="ts">
  import { onMount } from "svelte";
  import { fly } from "svelte/transition";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import FocusView from "$lib/components/FocusView.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import Sidebar, { type AppPage } from "$lib/components/Sidebar.svelte";
  import StopwatchCard from "$lib/components/StopwatchCard.svelte";
  import TimerCard from "$lib/components/TimerCard.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { stopwatchStore } from "$lib/stores/stopwatches.svelte";
  import { timerStore } from "$lib/stores/timers.svelte";

  const appWindow = getCurrentWindow();

  let activePage = $state<AppPage>("stopwatch");
  let isSidebarCollapsed = $state(false);
  let isCreating = $state(false);
  let isMaximized = $state(false);
  let isEditMode = $state(false);

  let activeErrorMessage = $derived(
    activePage === "stopwatch"
      ? stopwatchStore.errorMessage
      : activePage === "timer"
        ? timerStore.errorMessage
        : null
  );

  onMount(() => {
    void stopwatchStore.load();
    void timerStore.load();

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
      }
    } finally {
      isCreating = false;
    }
  }

  function handleSelectPage(page: AppPage): void {
    activePage = page;
    isEditMode = false;
  }

  function handleDismissError(): void {
    if (activePage === "stopwatch") {
      stopwatchStore.clearError();
    } else {
      timerStore.clearError();
    }
  }

  async function handleFocusStopwatch(id: number): Promise<void> {
    await stopwatchStore.enterFocus(id);
  }
</script>

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
                />
              {/each}
            </section>
          {/if}
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

    {#if activePage !== "settings"}
      <div
        class="fab-bar"
        role="toolbar"
        aria-label={activePage === "stopwatch" ? "Stopwatch actions" : "Timer actions"}
      >
      <button
        type="button"
        class="fab-btn"
        class:fab-btn-active={isEditMode}
        onclick={() => (isEditMode = !isEditMode)}
        aria-pressed={isEditMode}
        aria-label={isEditMode
          ? "Done editing"
          : activePage === "stopwatch"
            ? "Edit stopwatches"
            : "Edit timers"}
        title={isEditMode
          ? "Done editing"
          : activePage === "stopwatch"
            ? "Edit stopwatches"
            : "Edit timers"}
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
        aria-label={activePage === "stopwatch" ? "New stopwatch" : "New timer"}
        title={activePage === "stopwatch" ? "New stopwatch" : "New timer"}
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

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100vh;
    background-color: transparent;
    color: var(--text-primary);
  }

  .titlebar {
    height: 36px;
    padding-left: 12px;
    background-color: var(--bg-app);
    display: flex;
    align-items: center;
    justify-content: space-between;
    user-select: none;
    flex-shrink: 0;
  }

  .titlebar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .titlebar-toggle {
    width: 28px;
    height: 28px;
    border: none;
    border-radius: 6px;
    background-color: transparent;
    color: var(--text-muted);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      color 120ms ease;
  }

  .titlebar-toggle:hover {
    background-color: var(--bg-hover);
    color: var(--text-strong);
  }

  .titlebar-brand {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .titlebar-logo {
    width: 18px;
    height: 18px;
    object-fit: contain;
    flex-shrink: 0;
    pointer-events: none;
  }

  .titlebar-title {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-primary);
    letter-spacing: -0.005em;
  }

  .titlebar-controls {
    display: flex;
    align-items: stretch;
    height: 100%;
    flex-shrink: 0;
  }

  .win-btn {
    width: 46px;
    height: 100%;
    border: none;
    background-color: transparent;
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      background-color 100ms ease,
      color 100ms ease;
  }

  .win-btn:hover {
    background-color: var(--bg-hover);
    color: var(--text-strong);
  }

  .win-btn-close:hover {
    background-color: #c42b1c;
    color: #ffffff;
  }

  .error-banner {
    margin: 12px 16px 0;
    padding: 10px 12px;
    background-color: var(--bg-card);
    border: 1px solid var(--border-card);
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    flex-shrink: 0;
  }

  .error-text {
    font-size: 0.8125rem;
    color: var(--danger-fg);
  }

  .btn-dismiss {
    background-color: var(--bg-control);
    color: var(--text-secondary);
    border: 1px solid var(--border-control);
    border-radius: 6px;
    padding: 4px 10px;
    font-family: inherit;
    font-size: 0.75rem;
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      color 120ms ease;
  }

  .btn-dismiss:hover {
    background-color: var(--bg-control-hover);
    color: var(--text-strong);
  }

  .notification-banner {
    position: absolute;
    top: 12px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 25;
    padding: 10px 14px;
    min-width: 280px;
    max-width: calc(100% - 32px);
    background-color: var(--bg-card);
    border: 1px solid var(--border-strong);
    border-radius: 10px;
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .notification-icon {
    color: var(--text-primary);
    flex-shrink: 0;
  }

  .notification-copy {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .notification-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .notification-sub {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .btn-stop-sound {
    background-color: var(--accent-bg);
    color: var(--accent-fg);
    border: 1px solid var(--accent-bg);
    border-radius: 6px;
    padding: 6px 12px;
    font-family: inherit;
    font-size: 0.75rem;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      border-color 120ms ease;
  }

  .btn-stop-sound:hover {
    background-color: var(--accent-bg-hover);
    border-color: var(--accent-bg-hover);
  }

  .workspace {
    display: flex;
    flex: 1;
    min-height: 0;
    min-width: 0;
    border-top: 1px solid var(--border-subtle);
  }

  .content-area {
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: var(--bg-app);
    min-width: 0;
    min-height: 0;
  }

  .stopwatch-grid {
    flex: 1;
    overflow-y: auto;
    padding: 16px 20px 76px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 300px), 1fr));
    align-content: start;
    gap: 16px;
  }

  .fab-bar {
    position: fixed;
    right: 20px;
    bottom: 20px;
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px;
    background-color: var(--bg-fab);
    border: 1px solid var(--border-control);
    border-radius: 8px;
    z-index: 20;
  }

  .fab-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 6px;
    background-color: transparent;
    color: var(--text-primary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      color 120ms ease;
  }

  .fab-btn:hover:not(:disabled) {
    background-color: var(--bg-control-hover);
    color: var(--text-strong);
  }

  .fab-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .fab-btn-active {
    background-color: var(--bg-control);
    color: var(--text-strong);
  }

  .fab-btn-active:hover:not(:disabled) {
    background-color: var(--bg-control-hover);
  }

  .empty-state {
    flex: 1;
    padding: 32px 20px 64px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    text-align: center;
    user-select: none;
  }

  .empty-icon {
    color: var(--text-secondary);
    margin-bottom: 4px;
  }

  .empty-title {
    margin: 0;
    font-size: 1.0625rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .empty-subtext {
    margin: 0;
    font-size: 0.8125rem;
    color: var(--text-muted);
  }
</style>
