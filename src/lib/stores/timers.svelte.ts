import {
  type TimerDto,
  listTimers,
  createTimer,
  renameTimer,
  setTimerDuration,
  startTimer,
  pauseTimer,
  completeTimer,
  resetTimer,
  deleteTimer,
} from "../api";
import { computeRemainingMs } from "../time";
import { playTimerChime } from "../sound";

class TimerStore {
  items = $state<TimerDto[]>([]);
  loading = $state<boolean>(true);
  errorMessage = $state<string | null>(null);
  nowMs = $state<number>(Date.now());

  #intervalId: ReturnType<typeof setInterval> | null = null;
  #completingIds = new Set<number>();

  hasRunning = $derived(this.items.some((item) => item.status === "running"));
  runningCount = $derived(this.items.filter((item) => item.status === "running").length);
  completedCount = $derived(this.items.filter((item) => item.status === "completed").length);

  #syncTicker(): void {
    const anyRunning = this.items.some((item) => item.status === "running");
    if (anyRunning && this.#intervalId === null) {
      this.nowMs = Date.now();
      this.#intervalId = setInterval(() => {
        this.nowMs = Date.now();
        this.#checkCompletions();
      }, 100);
    } else if (!anyRunning && this.#intervalId !== null) {
      clearInterval(this.#intervalId);
      this.#intervalId = null;
      this.nowMs = Date.now();
    }
  }

  #checkCompletions(): void {
    for (const item of this.items) {
      if (item.status === "running" && !this.#completingIds.has(item.id)) {
        const remaining = computeRemainingMs(item, this.nowMs);
        if (remaining <= 0) {
          void this.#handleAutoComplete(item.id);
        }
      }
    }
  }

  async #handleAutoComplete(id: number): Promise<void> {
    if (this.#completingIds.has(id)) return;
    this.#completingIds.add(id);
    try {
      const updated = await completeTimer(id);
      this.#replaceItem(updated);
      playTimerChime();
    } catch (err) {
      this.setError(err);
    } finally {
      this.#completingIds.delete(id);
    }
  }

  #replaceItem(updated: TimerDto): void {
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
      this.items = await listTimers();
      this.nowMs = Date.now();
      this.#syncTicker();
    } catch (err) {
      this.setError(err);
    } finally {
      this.loading = false;
    }
  }

  async create(name?: string, durationMs?: number): Promise<TimerDto | null> {
    try {
      const created = await createTimer(name, durationMs);
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
      const updated = await renameTimer(id, name);
      this.#replaceItem(updated);
      return true;
    } catch (err) {
      this.setError(err);
      return false;
    }
  }

  async setDuration(id: number, durationMs: number): Promise<boolean> {
    try {
      const updated = await setTimerDuration(id, durationMs);
      this.#replaceItem(updated);
      return true;
    } catch (err) {
      this.setError(err);
      return false;
    }
  }

  async start(id: number): Promise<void> {
    try {
      const updated = await startTimer(id);
      this.#replaceItem(updated);
    } catch (err) {
      this.setError(err);
    }
  }

  async pause(id: number): Promise<void> {
    try {
      const updated = await pauseTimer(id);
      this.#replaceItem(updated);
    } catch (err) {
      this.setError(err);
    }
  }

  async reset(id: number): Promise<void> {
    try {
      const updated = await resetTimer(id);
      this.#replaceItem(updated);
    } catch (err) {
      this.setError(err);
    }
  }

  async remove(id: number): Promise<void> {
    try {
      await deleteTimer(id);
      this.items = this.items.filter((it) => it.id !== id);
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

export const timerStore = new TimerStore();
