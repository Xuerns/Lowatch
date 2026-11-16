import {
  type StopwatchDto,
  listStopwatches,
  createStopwatch,
  renameStopwatch,
  startStopwatch,
  pauseStopwatch,
  resetStopwatch,
  deleteStopwatch,
} from "../api";

class StopwatchStore {
  items = $state<StopwatchDto[]>([]);
  loading = $state<boolean>(true);
  errorMessage = $state<string | null>(null);
  nowMs = $state<number>(Date.now());
  focusedId = $state<number | null>(null);

  #intervalId: ReturnType<typeof setInterval> | null = null;

  hasRunning = $derived(this.items.some((item) => item.status === "running"));
  runningCount = $derived(this.items.filter((item) => item.status === "running").length);

  /**
   * Mengatur hidup/mati ticker global 100ms.
   * Dipanggil setiap kali daftar `items` berubah.
   * Ticker HANYA aktif apabila minimal ada 1 stopwatch dengan `status === "running"`.
   */
  #syncTicker(): void {
    const anyRunning = this.items.some((item) => item.status === "running");
    if (anyRunning && this.#intervalId === null) {
      this.nowMs = Date.now();
      this.#intervalId = setInterval(() => {
        this.nowMs = Date.now();
      }, 100);
    } else if (!anyRunning && this.#intervalId !== null) {
      clearInterval(this.#intervalId);
      this.#intervalId = null;
      this.nowMs = Date.now();
    }
  }

  #replaceItem(updated: StopwatchDto): void {
    const idx = this.items.findIndex((it) => it.id === updated.id);
    if (idx !== -1) {
      this.items[idx] = updated;
    } else {
      this.items = [...this.items, updated].sort(
        (a, b) => a.position - b.position || a.id - b.id
      );
    }
    this.nowMs = Date.now();
    this.#syncTicker();
  }

  clearError(): void {
    this.errorMessage = null;
  }

  setError(msg: unknown): void {
    this.errorMessage = typeof msg === "string" ? msg : String(msg);
  }

  async load(): Promise<void> {
    this.loading = true;
    try {
      this.items = await listStopwatches();
      this.nowMs = Date.now();
      this.#syncTicker();
    } catch (err) {
      this.setError(err);
    } finally {
      this.loading = false;
    }
  }

  async create(name?: string): Promise<StopwatchDto | null> {
    try {
      const created = await createStopwatch(name);
      this.items = [...this.items, created];
      this.nowMs = Date.now();
      this.#syncTicker();
      return created;
    } catch (err) {
      this.setError(err);
      return null;
    }
  }

  async rename(id: number, name: string): Promise<boolean> {
    try {
      const updated = await renameStopwatch(id, name);
      this.#replaceItem(updated);
      return true;
    } catch (err) {
      this.setError(err);
      return false;
    }
  }

  async start(id: number): Promise<void> {
    try {
      const updated = await startStopwatch(id);
      this.#replaceItem(updated);
    } catch (err) {
      this.setError(err);
    }
  }

  async pause(id: number): Promise<void> {
    try {
      const updated = await pauseStopwatch(id);
      this.#replaceItem(updated);
    } catch (err) {
      this.setError(err);
    }
  }

  async reset(id: number): Promise<void> {
    try {
      const updated = await resetStopwatch(id);
      this.#replaceItem(updated);
    } catch (err) {
      this.setError(err);
    }
  }

  async remove(id: number): Promise<void> {
    try {
      await deleteStopwatch(id);
      this.items = this.items.filter((it) => it.id !== id);
      if (this.focusedId === id) {
        this.focusedId = null;
      }
      this.#syncTicker();
    } catch (err) {
      this.setError(err);
    }
  }

  destroy(): void {
    if (this.#intervalId !== null) {
      clearInterval(this.#intervalId);
      this.#intervalId = null;
    }
  }
}

export const stopwatchStore = new StopwatchStore();
