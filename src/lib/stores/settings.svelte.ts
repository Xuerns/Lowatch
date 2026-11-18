import {
  clearCustomRingtone,
  extractWaveformPeaks,
  loadCustomRingtone,
  saveCustomRingtone,
  trimAudioFileToWav,
  validateAndInspectAudioFile,
} from "../ringtoneStorage";
import {
  playAudioRangePreview,
  playCustomRingtone,
  playTimerChime,
  stopCustomRingtone,
} from "../sound";

const STORAGE_KEY = "lowatch.settings.v1";

export type ThemeMode = "dark" | "light" | "system";

export const DIAL_STYLES = ["classic", "sunburst", "comet", "dots", "gauge"] as const;
export type DialStyle = (typeof DIAL_STYLES)[number];

export interface AppSettings {
  theme: ThemeMode;
  transparentWindow: boolean;
  windowOpacity: number;
  timerSoundEnabled: boolean;
  showStopwatchTenths: boolean;
  confirmStopwatchReset: boolean;
  defaultTimerDurationMs: number;
  dialStyle: DialStyle;
}

const DEFAULT_SETTINGS: AppSettings = {
  theme: "dark",
  transparentWindow: false,
  windowOpacity: 0.78,
  timerSoundEnabled: true,
  showStopwatchTenths: true,
  confirmStopwatchReset: true,
  defaultTimerDurationMs: 5 * 60_000,
  dialStyle: "classic",
};

function isValidTheme(val: unknown): val is ThemeMode {
  return val === "dark" || val === "light" || val === "system";
}

function isValidDialStyle(val: unknown): val is DialStyle {
  return typeof val === "string" && (DIAL_STYLES as readonly string[]).includes(val);
}

function loadInitialSettings(): AppSettings {
  if (typeof window === "undefined") return { ...DEFAULT_SETTINGS };
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_SETTINGS };
    const parsed = JSON.parse(raw) as Partial<AppSettings>;
    return {
      theme: isValidTheme(parsed.theme) ? parsed.theme : DEFAULT_SETTINGS.theme,
      transparentWindow:
        typeof parsed.transparentWindow === "boolean"
          ? parsed.transparentWindow
          : DEFAULT_SETTINGS.transparentWindow,
      windowOpacity:
        typeof parsed.windowOpacity === "number" &&
        parsed.windowOpacity >= 0.3 &&
        parsed.windowOpacity <= 0.95
          ? parsed.windowOpacity
          : DEFAULT_SETTINGS.windowOpacity,
      timerSoundEnabled:
        typeof parsed.timerSoundEnabled === "boolean"
          ? parsed.timerSoundEnabled
          : DEFAULT_SETTINGS.timerSoundEnabled,
      showStopwatchTenths:
        typeof parsed.showStopwatchTenths === "boolean"
          ? parsed.showStopwatchTenths
          : DEFAULT_SETTINGS.showStopwatchTenths,
      confirmStopwatchReset:
        typeof parsed.confirmStopwatchReset === "boolean"
          ? parsed.confirmStopwatchReset
          : DEFAULT_SETTINGS.confirmStopwatchReset,
      defaultTimerDurationMs:
        typeof parsed.defaultTimerDurationMs === "number" &&
        parsed.defaultTimerDurationMs >= 1_000
          ? parsed.defaultTimerDurationMs
          : DEFAULT_SETTINGS.defaultTimerDurationMs,
      dialStyle: isValidDialStyle(parsed.dialStyle)
        ? parsed.dialStyle
        : DEFAULT_SETTINGS.dialStyle,
    };
  } catch {
    return { ...DEFAULT_SETTINGS };
  }
}

class SettingsStore {
  theme = $state<ThemeMode>(DEFAULT_SETTINGS.theme);
  transparentWindow = $state<boolean>(DEFAULT_SETTINGS.transparentWindow);
  windowOpacity = $state<number>(DEFAULT_SETTINGS.windowOpacity);
  timerSoundEnabled = $state<boolean>(DEFAULT_SETTINGS.timerSoundEnabled);
  showStopwatchTenths = $state<boolean>(DEFAULT_SETTINGS.showStopwatchTenths);
  confirmStopwatchReset = $state<boolean>(DEFAULT_SETTINGS.confirmStopwatchReset);
  defaultTimerDurationMs = $state<number>(DEFAULT_SETTINGS.defaultTimerDurationMs);
  dialStyle = $state<DialStyle>(DEFAULT_SETTINGS.dialStyle);
  customRingtoneName = $state<string | null>(null);
  customRingtoneDurationSec = $state<number | null>(null);
  customRingtoneSize = $state<number | null>(null);
  ringtoneError = $state<string | null>(null);
  isRingtoneLoading = $state<boolean>(false);
  isPreviewPlaying = $state<boolean>(false);
  isNotificationPlaying = $state<boolean>(false);
  activeNotificationTimerName = $state<string | null>(null);
  pendingTrimFile = $state<File | null>(null);
  pendingTrimDurationSec = $state<number>(0);
  pendingTrimPeaks = $state<number[]>([]);
  isTrimPreviewPlaying = $state<boolean>(false);
  isTrimSaving = $state<boolean>(false);

  #customRingtoneBlob: Blob | null = null;

  constructor() {
    const initial = loadInitialSettings();
    this.theme = initial.theme;
    this.transparentWindow = initial.transparentWindow;
    this.windowOpacity = initial.windowOpacity;
    this.timerSoundEnabled = initial.timerSoundEnabled;
    this.showStopwatchTenths = initial.showStopwatchTenths;
    this.confirmStopwatchReset = initial.confirmStopwatchReset;
    this.defaultTimerDurationMs = initial.defaultTimerDurationMs;
    this.dialStyle = initial.dialStyle;
    this.#applyTheme();

    if (typeof window !== "undefined") {
      void this.#loadRingtone();
      if (typeof window.matchMedia === "function") {
        const mq = window.matchMedia("(prefers-color-scheme: light)");
        mq.addEventListener("change", () => {
          if (this.theme === "system") {
            this.#applyTheme();
          }
        });
      }
    }
  }

  async #loadRingtone(): Promise<void> {
    try {
      const rec = await loadCustomRingtone();
      if (rec) {
        this.customRingtoneName = rec.name;
        this.customRingtoneDurationSec = rec.durationSec;
        this.customRingtoneSize = rec.size;
        this.#customRingtoneBlob = rec.blob;
      }
    } catch {
      // Ignore IndexedDB unavailability
    }
  }

  #applyTheme(): void {
    if (typeof document === "undefined") return;
    let resolved: "dark" | "light" = "dark";
    if (this.theme === "light") {
      resolved = "light";
    } else if (
      this.theme === "system" &&
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-color-scheme: light)").matches
    ) {
      resolved = "light";
    }
    document.documentElement.dataset.theme = resolved;
    document.documentElement.dataset.transparent = this.transparentWindow ? "true" : "false";
    document.documentElement.style.colorScheme = resolved;
    const windowAlpha = this.transparentWindow ? this.windowOpacity : 1;
    const cardAlpha = this.transparentWindow ? Math.min(0.95, this.windowOpacity + 0.12) : 1;
    document.documentElement.style.setProperty("--window-alpha", String(windowAlpha));
    document.documentElement.style.setProperty("--card-alpha", String(cardAlpha));
  }

  #persist(): void {
    if (typeof window === "undefined") return;
    try {
      const payload: AppSettings = {
        theme: this.theme,
        transparentWindow: this.transparentWindow,
        windowOpacity: this.windowOpacity,
        timerSoundEnabled: this.timerSoundEnabled,
        showStopwatchTenths: this.showStopwatchTenths,
        confirmStopwatchReset: this.confirmStopwatchReset,
        defaultTimerDurationMs: this.defaultTimerDurationMs,
        dialStyle: this.dialStyle,
      };
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify(payload));
    } catch {
      // Ignore storage quota / privacy mode errors
    }
  }

  setTheme(theme: ThemeMode): void {
    this.theme = theme;
    this.#applyTheme();
    this.#persist();
  }

  setTransparentWindow(enabled: boolean): void {
    this.transparentWindow = enabled;
    this.#applyTheme();
    this.#persist();
  }

  setWindowOpacity(opacity: number): void {
    const clamped = Math.max(0.3, Math.min(0.95, Math.round(opacity * 100) / 100));
    this.windowOpacity = clamped;
    this.#applyTheme();
    this.#persist();
  }

  setTimerSoundEnabled(enabled: boolean): void {
    this.timerSoundEnabled = enabled;
    this.#persist();
  }

  setShowStopwatchTenths(show: boolean): void {
    this.showStopwatchTenths = show;
    this.#persist();
  }

  setConfirmStopwatchReset(confirm: boolean): void {
    this.confirmStopwatchReset = confirm;
    this.#persist();
  }

  setDefaultTimerDurationMs(durationMs: number): void {
    if (durationMs < 1_000) return;
    this.defaultTimerDurationMs = durationMs;
    this.#persist();
  }

  setDialStyle(style: DialStyle): void {
    this.dialStyle = style;
    this.#persist();
  }

  clearRingtoneError(): void {
    this.ringtoneError = null;
  }

  async uploadCustomRingtone(file: File): Promise<"saved" | "needs_trim" | "error"> {
    this.ringtoneError = null;
    this.isRingtoneLoading = true;
    this.stopPreview();
    this.stopTrimPreview();
    try {
      const { durationSec, needsTrim } = await validateAndInspectAudioFile(file);
      if (needsTrim) {
        this.pendingTrimFile = file;
        this.pendingTrimDurationSec = durationSec;
        this.pendingTrimPeaks = await extractWaveformPeaks(file, 96);
        return "needs_trim";
      }
      const saved = await saveCustomRingtone(file, durationSec);
      this.customRingtoneName = saved.name;
      this.customRingtoneDurationSec = saved.durationSec;
      this.customRingtoneSize = saved.size;
      this.#customRingtoneBlob = saved.blob;
      this.pendingTrimFile = null;
      this.pendingTrimDurationSec = 0;
      this.pendingTrimPeaks = [];
      return "saved";
    } catch (err) {
      this.ringtoneError = err instanceof Error ? err.message : String(err);
      return "error";
    } finally {
      this.isRingtoneLoading = false;
    }
  }

  cancelTrim(): void {
    this.stopTrimPreview();
    this.pendingTrimFile = null;
    this.pendingTrimDurationSec = 0;
    this.pendingTrimPeaks = [];
    this.ringtoneError = null;
  }

  stopTrimPreview(): void {
    if (this.isTrimPreviewPlaying) {
      stopCustomRingtone();
      this.isTrimPreviewPlaying = false;
    }
  }

  toggleTrimPreview(startSec: number, endSec: number): void {
    if (this.isTrimPreviewPlaying) {
      this.stopTrimPreview();
      return;
    }
    if (!this.pendingTrimFile) return;
    this.stopPreview();
    this.isTrimPreviewPlaying = true;
    playAudioRangePreview(this.pendingTrimFile, startSec, endSec, () => {
      this.isTrimPreviewPlaying = false;
    });
  }

  async confirmTrimAndSave(startSec: number, endSec: number): Promise<boolean> {
    if (!this.pendingTrimFile || this.isTrimSaving) return false;
    this.ringtoneError = null;
    this.isTrimSaving = true;
    this.stopTrimPreview();
    try {
      const { trimmedFile, durationSec } = await trimAudioFileToWav(
        this.pendingTrimFile,
        startSec,
        endSec
      );
      const saved = await saveCustomRingtone(trimmedFile, durationSec);
      this.customRingtoneName = saved.name;
      this.customRingtoneDurationSec = saved.durationSec;
      this.customRingtoneSize = saved.size;
      this.#customRingtoneBlob = saved.blob;
      this.pendingTrimFile = null;
      this.pendingTrimDurationSec = 0;
      this.pendingTrimPeaks = [];
      return true;
    } catch (err) {
      this.ringtoneError = err instanceof Error ? err.message : String(err);
      return false;
    } finally {
      this.isTrimSaving = false;
    }
  }

  async resetCustomRingtone(): Promise<void> {
    this.ringtoneError = null;
    this.stopPreview();
    try {
      await clearCustomRingtone();
    } catch {
      // Ignore storage clear errors
    } finally {
      this.customRingtoneName = null;
      this.customRingtoneDurationSec = null;
      this.customRingtoneSize = null;
      this.#customRingtoneBlob = null;
    }
  }

  stopPreview(): void {
    stopCustomRingtone();
    this.isPreviewPlaying = false;
  }

  togglePreviewRingtone(): void {
    if (this.isPreviewPlaying) {
      this.stopPreview();
      return;
    }
    if (this.#customRingtoneBlob) {
      this.isPreviewPlaying = true;
      playCustomRingtone(this.#customRingtoneBlob, () => {
        this.isPreviewPlaying = false;
      });
    } else {
      playTimerChime();
    }
  }

  stopTimerNotification(): void {
    stopCustomRingtone();
    this.isNotificationPlaying = false;
    this.activeNotificationTimerName = null;
  }

  playTimerNotification(timerName?: string): void {
    if (!this.timerSoundEnabled) return;
    this.stopPreview();
    this.stopTrimPreview();
    if (this.#customRingtoneBlob) {
      this.isNotificationPlaying = true;
      this.activeNotificationTimerName = timerName ?? null;
      playCustomRingtone(this.#customRingtoneBlob, () => {
        this.isNotificationPlaying = false;
        this.activeNotificationTimerName = null;
      });
    } else {
      playTimerChime();
    }
  }
}

export const settingsStore = new SettingsStore();
