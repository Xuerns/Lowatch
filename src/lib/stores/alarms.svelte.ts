import {
  type AlarmDto,
  bringWindowToFront,
  createAlarm,
  deleteAlarm,
  dismissAlarm,
  listAlarms,
  ringAlarm,
  setAlarmEnabled,
  snoozeAlarm,
  updateAlarm,
} from "../api";
import { settingsStore } from "./settings.svelte";

class AlarmStore {
  items = $state<AlarmDto[]>([]);
  loading = $state<boolean>(true);
  errorMessage = $state<string | null>(null);
  nowMs = $state<number>(Date.now());
  ringingAlarm = $state<AlarmDto | null>(null);

  activeCount = $derived(this.items.filter((item) => item.enabled).length);

  #intervalId: ReturnType<typeof setInterval> | null = null;
  #ringingIds = new Set<number>();
  #isHandlingDue = false;
  #isResolvingRing = false;

  #syncTicker(): void {
    const anyEnabled = this.items.some((item) => item.enabled);
    const shouldTick = anyEnabled || this.ringingAlarm !== null;
    if (shouldTick && this.#intervalId === null) {
      this.nowMs = Date.now();
      this.#intervalId = setInterval(() => {
        this.nowMs = Date.now();
        this.#checkDue();
      }, 1_000);
    } else if (!shouldTick && this.#intervalId !== null) {
      clearInterval(this.#intervalId);
      this.#intervalId = null;
      this.nowMs = Date.now();
    }
  }

  #checkDue(): void {
    if (this.ringingAlarm !== null || this.#isHandlingDue) return;
    const now = Date.now();
    const due = this.items.find(
      (item) =>
        item.enabled &&
        item.next_ring_at_ms !== null &&
        item.next_ring_at_ms <= now &&
        !this.#ringingIds.has(item.id)
    );
    if (due) {
      void this.#handleDue(due.id);
    }
  }

  async #handleDue(id: number): Promise<void> {
    if (this.ringingAlarm !== null || this.#ringingIds.has(id)) return;
    this.#ringingIds.add(id);
    this.#isHandlingDue = true;
    try {
      const updated = await ringAlarm(id);
      this.ringingAlarm = updated;
      this.#replaceItem(updated);
      settingsStore.playAlarmNotification();
      void bringWindowToFront().catch(() => {
        // Ignore outside Tauri runtime (e.g. browser preview)
      });
    } catch (err) {
      this.setError(err);
    } finally {
      this.#isHandlingDue = false;
      this.#ringingIds.delete(id);
    }
  }

  #replaceItem(updated: AlarmDto): void {
    const index = this.items.findIndex((item) => item.id === updated.id);
    if (index !== -1) {
      this.items[index] = updated;
    } else {
      this.items = [...this.items, updated];
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
      this.items = await listAlarms();
      this.nowMs = Date.now();
      this.#syncTicker();
      this.#checkDue();
    } catch (err) {
      this.setError(err);
    } finally {
      this.loading = false;
    }
  }

  async create(): Promise<AlarmDto | null> {
    try {
      const created = await createAlarm();
      this.items = [...this.items, created];
      this.nowMs = Date.now();
      this.#syncTicker();
      return created;
    } catch (err) {
      this.setError(err);
      return null;
    }
  }

  async update(
    id: number,
    name: string,
    hour: number,
    minute: number,
    repeatDays: number
  ): Promise<boolean> {
    try {
      const updated = await updateAlarm(id, name, hour, minute, repeatDays);
      this.#replaceItem(updated);
      return true;
    } catch (err) {
      this.setError(err);
      return false;
    }
  }

  async toggle(id: number, enabled: boolean): Promise<void> {
    try {
      const updated = await setAlarmEnabled(id, enabled);
      this.#replaceItem(updated);
      if (!enabled && this.ringingAlarm?.id === id) {
        this.ringingAlarm = null;
        settingsStore.stopAlarmNotification();
        this.#syncTicker();
      }
    } catch (err) {
      this.setError(err);
    }
  }

  async remove(id: number): Promise<void> {
    try {
      await deleteAlarm(id);
      this.items = this.items.filter((item) => item.id !== id);
      if (this.ringingAlarm?.id === id) {
        this.ringingAlarm = null;
        settingsStore.stopAlarmNotification();
      }
      this.nowMs = Date.now();
      this.#syncTicker();
      this.#checkDue();
    } catch (err) {
      this.setError(err);
    }
  }

  async snooze(id: number): Promise<void> {
    if (this.#isResolvingRing) return;
    this.#isResolvingRing = true;
    try {
      const updated = await snoozeAlarm(id, settingsStore.alarmSnoozeMinutes);
      this.#replaceItem(updated);
      this.ringingAlarm = null;
      settingsStore.stopAlarmNotification();
      this.#syncTicker();
      this.#checkDue();
    } catch (err) {
      this.setError(err);
    } finally {
      this.#isResolvingRing = false;
    }
  }

  async dismiss(id: number): Promise<void> {
    if (this.#isResolvingRing) return;
    this.#isResolvingRing = true;
    try {
      const updated = await dismissAlarm(id);
      this.#replaceItem(updated);
      this.ringingAlarm = null;
      settingsStore.stopAlarmNotification();
      this.#syncTicker();
      this.#checkDue();
    } catch (err) {
      this.setError(err);
    } finally {
      this.#isResolvingRing = false;
    }
  }

  destroy(): void {
    if (this.#intervalId !== null) {
      clearInterval(this.#intervalId);
      this.#intervalId = null;
    }
    settingsStore.stopAlarmNotification();
  }
}

export const alarmStore = new AlarmStore();
