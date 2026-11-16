<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import FocusView from "$lib/components/FocusView.svelte";
  import StopwatchCard from "$lib/components/StopwatchCard.svelte";
  import { stopwatchStore } from "$lib/stores/stopwatches.svelte";

  const appWindow = getCurrentWindow();

  let isCreating = $state(false);
  let isMaximized = $state(false);
  let isEditMode = $state(false);

  onMount(() => {
    void stopwatchStore.load();

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
      await stopwatchStore.create();
    } finally {
      isCreating = false;
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
      <div class="titlebar-brand" data-tauri-drag-region>
        <img src="/logo.png" alt="Lowatch logo" class="titlebar-logo" />
        <span class="titlebar-title" data-tauri-drag-region>Lowatch</span>
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

    {#if stopwatchStore.errorMessage !== null}
      <div class="error-banner" role="alert">
        <span class="error-text">{stopwatchStore.errorMessage}</span>
        <button
          type="button"
          class="btn-dismiss"
          onclick={() => stopwatchStore.clearError()}
        >
          Dismiss
        </button>
      </div>
    {/if}

    <section class="stopwatch-grid" aria-label="Stopwatch list">
      {#if stopwatchStore.loading && stopwatchStore.items.length === 0}
        <div class="empty-state">
          <p class="empty-text">Loading stopwatches...</p>
        </div>
      {:else if stopwatchStore.items.length === 0}
        <div class="empty-state">
          <p class="empty-text">No stopwatches yet</p>
          <button
            type="button"
            class="btn-empty-create"
            disabled={isCreating}
            onclick={handleQuickCreate}
          >
            + Create stopwatch
          </button>
        </div>
      {:else}
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
      {/if}
    </section>

    <div class="fab-bar" role="toolbar" aria-label="Stopwatch actions">
      <button
        type="button"
        class="fab-btn"
        class:fab-btn-active={isEditMode}
        onclick={() => (isEditMode = !isEditMode)}
        aria-pressed={isEditMode}
        aria-label={isEditMode ? "Done editing" : "Edit stopwatches"}
        title={isEditMode ? "Done editing" : "Edit stopwatches"}
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
        aria-label="New stopwatch"
        title="New stopwatch"
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
  </main>
{/if}

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100vh;
    background-color: #202020;
    color: #f5f5f5;
  }

  .titlebar {
    height: 36px;
    padding-left: 12px;
    background-color: #202020;
    display: flex;
    align-items: center;
    justify-content: space-between;
    user-select: none;
    flex-shrink: 0;
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
    color: #f5f5f5;
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
    color: #d4d4d4;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      background-color 100ms ease,
      color 100ms ease;
  }

  .win-btn:hover {
    background-color: rgba(255, 255, 255, 0.08);
    color: #ffffff;
  }

  .win-btn-close:hover {
    background-color: #c42b1c;
    color: #ffffff;
  }

  .error-banner {
    margin: 12px 16px 0;
    padding: 10px 12px;
    background-color: #2b2b2b;
    border: 1px solid #383838;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    flex-shrink: 0;
  }

  .error-text {
    font-size: 0.8125rem;
    color: #f87171;
  }

  .btn-dismiss {
    background-color: #323232;
    color: #d4d4d4;
    border: 1px solid #404040;
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
    background-color: #3d3d3d;
    color: #ffffff;
  }

  .stopwatch-grid {
    flex: 1;
    overflow-y: auto;
    padding: 8px 20px 76px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(min(100%, 310px), 1fr));
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
    background-color: #2d2d2d;
    border: 1px solid #424242;
    border-radius: 8px;
    z-index: 20;
  }

  .fab-btn {
    width: 36px;
    height: 36px;
    border: none;
    border-radius: 6px;
    background-color: transparent;
    color: #e5e5e5;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      color 120ms ease;
  }

  .fab-btn:hover:not(:disabled) {
    background-color: #3a3a3a;
    color: #ffffff;
  }

  .fab-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .fab-btn-active {
    background-color: #404040;
    color: #ffffff;
  }

  .fab-btn-active:hover:not(:disabled) {
    background-color: #4a4a4a;
  }

  .empty-state {
    grid-column: 1 / -1;
    margin: auto 0;
    padding: 32px 16px;
    border: 1px solid #383838;
    border-radius: 8px;
    background-color: #2b2b2b;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    text-align: center;
  }

  .empty-text {
    margin: 0;
    font-size: 0.9375rem;
    color: #b3b3b3;
  }

  .btn-empty-create {
    background-color: #f5f5f5;
    color: #202020;
    border: 1px solid #f5f5f5;
    border-radius: 6px;
    padding: 8px 16px;
    font-family: inherit;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    transition:
      background-color 120ms ease,
      border-color 120ms ease;
  }

  .btn-empty-create:hover:not(:disabled) {
    background-color: #e5e5e5;
    border-color: #e5e5e5;
  }

  .btn-empty-create:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
