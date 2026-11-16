<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();
</script>

{@render children()}

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  :global(html, body) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    background-color: #202020;
    color: #f5f5f5;
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
    background: #202020;
  }

  :global(*::-webkit-scrollbar-thumb) {
    background-color: #3a3a3a;
    border-radius: 4px;
    border: 2px solid #202020;
  }

  :global(*::-webkit-scrollbar-thumb:hover) {
    background-color: #525252;
  }

  :global(*::-webkit-scrollbar-thumb:active) {
    background-color: #666666;
  }

  :global(*::-webkit-scrollbar-corner) {
    background: #202020;
  }
</style>
