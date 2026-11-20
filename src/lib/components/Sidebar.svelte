<script lang="ts">
  export type AppPage = "stopwatch" | "timer" | "alarm" | "settings";

  interface Props {
    activePage: AppPage;
    collapsed: boolean;
    stopwatchRunningCount: number;
    timerRunningCount: number;
    alarmActiveCount?: number;
    onSelectPage: (page: AppPage) => void;
  }

  let {
    activePage,
    collapsed,
    stopwatchRunningCount,
    timerRunningCount,
    alarmActiveCount = 0,
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

    <button
      type="button"
      class="nav-item"
      class:nav-item-active={activePage === "alarm"}
      onclick={() => onSelectPage("alarm")}
      aria-current={activePage === "alarm" ? "page" : undefined}
      title="Alarms"
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
        <path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9" />
        <path d="M13.73 21a2 2 0 0 1-3.46 0" />
      </svg>
      {#if !collapsed}
        <span class="nav-label">Alarms</span>
        {#if alarmActiveCount > 0}
          <span class="nav-count" aria-label="{alarmActiveCount} active">
            {alarmActiveCount}
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

