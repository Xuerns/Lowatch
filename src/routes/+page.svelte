<script lang="ts">
  import { onMount } from "svelte";
  import FocusView from "$lib/components/FocusView.svelte";
  import StopwatchCard from "$lib/components/StopwatchCard.svelte";
  import { stopwatchStore } from "$lib/stores/stopwatches.svelte";

  let newName = $state("");
  let isCreating = $state(false);

  onMount(() => {
    void stopwatchStore.load();
    return () => {
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

  async function handleCreateSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (isCreating) return;
    isCreating = true;
    try {
      const created = await stopwatchStore.create(newName);
      if (created) {
        newName = "";
      }
    } finally {
      isCreating = false;
    }
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
    <header class="app-header">
      <div class="header-top">
        <h1 class="app-title">Multi Stopwatch</h1>
        <span class="running-counter">
          {#if stopwatchStore.runningCount > 0}
            {stopwatchStore.runningCount} berjalan
          {:else}
            {stopwatchStore.items.length} total
          {/if}
        </span>
      </div>

      <form class="create-form" onsubmit={handleCreateSubmit}>
        <input
          type="text"
          class="create-input"
          maxlength="50"
          placeholder="Nama stopwatch (opsional)..."
          bind:value={newName}
          aria-label="Nama stopwatch baru"
        />
        <button type="submit" class="btn-create" disabled={isCreating}>
          + Baru
        </button>
      </form>
    </header>

    {#if stopwatchStore.errorMessage !== null}
      <div class="error-banner" role="alert">
        <span class="error-text">{stopwatchStore.errorMessage}</span>
        <button
          type="button"
          class="btn-dismiss"
          onclick={() => stopwatchStore.clearError()}
        >
          Tutup
        </button>
      </div>
    {/if}

    <section class="stopwatch-list" aria-label="Daftar stopwatch">
      {#if stopwatchStore.loading && stopwatchStore.items.length === 0}
        <div class="empty-state">
          <p class="empty-text">Memuat stopwatch...</p>
        </div>
      {:else if stopwatchStore.items.length === 0}
        <div class="empty-state">
          <p class="empty-text">Belum ada stopwatch</p>
          <button
            type="button"
            class="btn-empty-create"
            disabled={isCreating}
            onclick={handleQuickCreate}
          >
            + Buat stopwatch
          </button>
        </div>
      {:else}
        {#each stopwatchStore.items as item (item.id)}
          <StopwatchCard
            stopwatch={item}
            nowMs={stopwatchStore.nowMs}
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
  </main>
{/if}

<style>
  .app-shell {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100vh;
    background-color: #0a0a0a;
    color: #f5f5f5;
  }

  .app-header {
    padding: 16px 16px 12px;
    border-bottom: 1px solid #262626;
    background-color: #0a0a0a;
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex-shrink: 0;
  }

  .header-top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
  }

  .app-title {
    margin: 0;
    font-size: 1.125rem;
    font-weight: 700;
    color: #f5f5f5;
    letter-spacing: -0.01em;
  }

  .running-counter {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.75rem;
    color: #a3a3a3;
  }

  .create-form {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .create-input {
    flex: 1;
    min-width: 0;
    background-color: #171717;
    color: #f5f5f5;
    border: 1px solid #262626;
    border-radius: 6px;
    padding: 8px 10px;
    font-size: 0.875rem;
    font-family: inherit;
    outline: none;
    transition: border-color 120ms ease;
  }

  .create-input::placeholder {
    color: #737373;
  }

  .create-input:focus-visible {
    border-color: #525252;
  }

  .btn-create {
    background-color: #f5f5f5;
    color: #0a0a0a;
    border: 1px solid #f5f5f5;
    border-radius: 6px;
    padding: 8px 14px;
    font-family: inherit;
    font-size: 0.875rem;
    font-weight: 600;
    cursor: pointer;
    flex-shrink: 0;
    transition:
      background-color 120ms ease,
      border-color 120ms ease;
  }

  .btn-create:hover:not(:disabled) {
    background-color: #e5e5e5;
    border-color: #e5e5e5;
  }

  .btn-create:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-banner {
    margin: 12px 16px 0;
    padding: 10px 12px;
    background-color: #171717;
    border: 1px solid #262626;
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
    background-color: #111111;
    color: #a3a3a3;
    border: 1px solid #262626;
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
    background-color: #1f1f1f;
    color: #f5f5f5;
  }

  .stopwatch-list {
    flex: 1;
    overflow-y: auto;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty-state {
    margin: auto 0;
    padding: 32px 16px;
    border: 1px solid #262626;
    border-radius: 8px;
    background-color: #111111;
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
    color: #a3a3a3;
  }

  .btn-empty-create {
    background-color: #f5f5f5;
    color: #0a0a0a;
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
