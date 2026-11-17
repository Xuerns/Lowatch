<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

{@render children()}

<style>
  :global(:root) {
    --window-alpha: 1;
    --card-alpha: 1;
    --bg-app: rgba(32, 32, 32, var(--window-alpha));
    --bg-sidebar: rgba(27, 27, 27, var(--window-alpha));
    --bg-card: rgba(43, 43, 43, var(--card-alpha));
    --bg-surface: rgba(36, 36, 36, var(--card-alpha));
    --bg-control: #323232;
    --bg-control-hover: #3c3c3c;
    --bg-hover: rgba(255, 255, 255, 0.08);
    --bg-active-nav: rgba(43, 43, 43, var(--card-alpha));
    --bg-active-nav-hover: rgba(48, 48, 48, var(--card-alpha));
    --bg-fab: rgba(45, 45, 45, var(--card-alpha));
    --bg-badge: #363636;
    --border-subtle: #303030;
    --border-card: #383838;
    --border-control: #404040;
    --border-strong: #525252;
    --text-primary: #f5f5f5;
    --text-strong: #ffffff;
    --text-secondary: #d4d4d4;
    --text-muted: #8a8a8a;
    --text-dim: #737373;
    --accent-bg: #f5f5f5;
    --accent-bg-hover: #e5e5e5;
    --accent-fg: #202020;
    --danger-fg: #f87171;
    --dial-tick: #363636;
    --dial-tick-major: #4a4a4a;
    --dial-tick-lit-paused: #666666;
    --dial-track: #303030;
    --dial-progress-paused: #5e5e5e;
    --dial-hand-idle: #4a4a4a;
    --dial-hand-paused: #7a7a7a;
    --scrollbar-thumb: #3a3a3a;
    --scrollbar-thumb-hover: #525252;
    --scrollbar-thumb-active: #666666;
  }

  :global(:root[data-theme="light"]) {
    --bg-app: rgba(245, 245, 245, var(--window-alpha));
    --bg-sidebar: rgba(236, 236, 236, var(--window-alpha));
    --bg-card: rgba(255, 255, 255, var(--card-alpha));
    --bg-surface: rgba(240, 240, 240, var(--card-alpha));
    --bg-control: #e8e8e8;
    --bg-control-hover: #dcdcdc;
    --bg-hover: rgba(0, 0, 0, 0.06);
    --bg-active-nav: rgba(255, 255, 255, var(--card-alpha));
    --bg-active-nav-hover: rgba(247, 247, 247, var(--card-alpha));
    --bg-fab: rgba(255, 255, 255, var(--card-alpha));
    --bg-badge: #dcdcdc;
    --border-subtle: #dcdcdc;
    --border-card: #d4d4d4;
    --border-control: #cccccc;
    --border-strong: #a3a3a3;
    --text-primary: #171717;
    --text-strong: #0a0a0a;
    --text-secondary: #404040;
    --text-muted: #666666;
    --text-dim: #737373;
    --accent-bg: #171717;
    --accent-bg-hover: #2e2e2e;
    --accent-fg: #ffffff;
    --danger-fg: #dc2626;
    --dial-tick: #d4d4d4;
    --dial-tick-major: #a3a3a3;
    --dial-tick-lit-paused: #737373;
    --dial-track: #e5e5e5;
    --dial-progress-paused: #a3a3a3;
    --dial-hand-idle: #a3a3a3;
    --dial-hand-paused: #737373;
    --scrollbar-thumb: #c4c4c4;
    --scrollbar-thumb-hover: #a3a3a3;
    --scrollbar-thumb-active: #8a8a8a;
  }

  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    background-color: transparent;
    color: var(--text-primary);
    color-scheme: dark;
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial,
      sans-serif;
    font-size: 16px;
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    overflow: hidden;
  }

  :global(*::-webkit-scrollbar) {
    width: 8px;
    height: 8px;
  }

  :global(*::-webkit-scrollbar-track) {
    background: var(--bg-app);
  }

  :global(*::-webkit-scrollbar-thumb) {
    background-color: var(--scrollbar-thumb);
    border-radius: 4px;
    border: 2px solid var(--bg-app);
  }

  :global(*::-webkit-scrollbar-thumb:hover) {
    background-color: var(--scrollbar-thumb-hover);
  }

  :global(*::-webkit-scrollbar-thumb:active) {
    background-color: var(--scrollbar-thumb-active);
  }

  :global(*::-webkit-scrollbar-corner) {
    background: var(--bg-app);
  }
</style>
