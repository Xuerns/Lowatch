<script lang="ts">
  export type AppPage = "stopwatch" | "timer" | "settings";

  interface Props {
    activePage: AppPage;
    collapsed: boolean;
    stopwatchRunningCount: number;
    timerRunningCount: number;
    onSelectPage: (page: AppPage) => void;
  }

  let {
    activePage,
    collapsed,
    stopwatchRunningCount,
    timerRunningCount,
    onSelectPage,
  }: Props = $props();
</script>

<aside class="sidebar" class:sidebar-collapsed={collapsed} aria-label="Navigation">
  <nav class="nav-list" aria-label="Primary pages">
    <button
      type="button"
      class="nav-item"
      class:nav-item-active={activePage === "stopwatch"}
      onclick={() => onSelectPage("stopwatch")}
      aria-current={activePage === "stopwatch" ? "page" : undefined}
      title="Stopwatch"
    >
      <svg
        class="nav-icon"
        width="17"
        height="17"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="13" r="8" />
        <path d="M12 9v4l2.5 1.5" />
        <path d="M10 2h4" />
        <path d="M12 2v3" />
      </svg>
      {#if !collapsed}
        <span class="nav-label">Stopwatch</span>
        {#if stopwatchRunningCount > 0}
          <span class="nav-count" aria-label="{stopwatchRunningCount} running">
            {stopwatchRunningCount}
          </span>
        {/if}
      {/if}
    </button>

    <button
      type="button"
      class="nav-item"
      class:nav-item-active={activePage === "timer"}
      onclick={() => onSelectPage("timer")}
      aria-current={activePage === "timer" ? "page" : undefined}
      title="Timer"
    >
      <svg
        class="nav-icon"
        width="17"
        height="17"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <path d="M5 2h14" />
        <path d="M5 22h14" />
        <path d="M7 2v4.5a5 5 0 0 0 1.46 3.54L12 13.5l3.54-3.46A5 5 0 0 0 17 6.5V2" />
        <path d="M7 22v-4.5a5 5 0 0 1 1.46-3.54L12 10.5l3.54 3.46A5 5 0 0 1 17 17.5V22" />
      </svg>
      {#if !collapsed}
        <span class="nav-label">Timer</span>
        {#if timerRunningCount > 0}
          <span class="nav-count" aria-label="{timerRunningCount} running">
            {timerRunningCount}
          </span>
        {/if}
      {/if}
    </button>
  </nav>

  <div class="sidebar-footer">
    <button
      type="button"
      class="nav-item"
      class:nav-item-active={activePage === "settings"}
      onclick={() => onSelectPage("settings")}
      aria-current={activePage === "settings" ? "page" : undefined}
      title="Settings"
    >
      <svg
        class="nav-icon"
        width="17"
        height="17"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="3" />
        <path
          d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z"
        />
      </svg>
      {#if !collapsed}
        <span class="nav-label">Settings</span>
      {/if}
    </button>
  </div>
</aside>

<style>
  .sidebar {
    width: 256px;
    flex-shrink: 0;
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    padding: 8px;
    gap: 8px;
    user-select: none;
    transition: width 150ms ease;
  }

  .sidebar-collapsed {
    width: 52px;
    align-items: center;
  }

  .nav-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    width: 100%;
    flex: 1;
  }

  .sidebar-footer {
    width: 100%;
    padding-top: 8px;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .nav-item {
    position: relative;
    width: 100%;
    height: 38px;
    padding: 0 10px;
    border: 1px solid transparent;
    border-radius: 6px;
    background-color: transparent;
    color: var(--text-muted);
    font-family: inherit;
    font-size: 0.875rem;
    font-weight: 500;
    display: flex;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    text-align: left;
    transition:
      background-color 120ms ease,
      color 120ms ease,
      border-color 120ms ease;
  }

  .sidebar-collapsed .nav-item {
    width: 36px;
    height: 36px;
    padding: 0;
    justify-content: center;
  }

  .nav-item:hover {
    background-color: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item-active {
    background-color: var(--bg-active-nav);
    border-color: var(--border-card);
    color: var(--text-strong);
  }

  .nav-item-active:hover {
    background-color: var(--bg-active-nav-hover);
  }

  .nav-icon {
    flex-shrink: 0;
  }

  .nav-label {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .nav-count {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-variant-numeric: tabular-nums;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
    background-color: var(--bg-badge);
    border-radius: 4px;
    padding: 1px 6px;
    line-height: 1.4;
    flex-shrink: 0;
  }
</style>
