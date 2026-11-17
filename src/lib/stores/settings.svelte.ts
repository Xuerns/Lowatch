const STORAGE_KEY = "lowatch.settings.v1";

export type ThemeMode = "dark" | "light" | "system";

export interface AppSettings {
  theme: ThemeMode;
  transparentWindow: boolean;
  windowOpacity: number;
  timerSoundEnabled: boolean;
  showStopwatchTenths: boolean;
  confirmStopwatchReset: boolean;
  defaultTimerDurationMs: number;
}

const DEFAULT_SETTINGS: AppSettings = {
  theme: "dark",
  transparentWindow: false,
  windowOpacity: 0.78,
  timerSoundEnabled: true,
  showStopwatchTenths: true,
  confirmStopwatchReset: true,
  defaultTimerDurationMs: 5 * 60_000,
};

function isValidTheme(val: unknown): val is ThemeMode {
  return val === "dark" || val === "light" || val === "system";
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

  constructor() {
    const initial = loadInitialSettings();
    this.theme = initial.theme;
    this.transparentWindow = initial.transparentWindow;
    this.windowOpacity = initial.windowOpacity;
    this.timerSoundEnabled = initial.timerSoundEnabled;
    this.showStopwatchTenths = initial.showStopwatchTenths;
    this.confirmStopwatchReset = initial.confirmStopwatchReset;
    this.defaultTimerDurationMs = initial.defaultTimerDurationMs;
    this.#applyTheme();

    if (typeof window !== "undefined" && typeof window.matchMedia === "function") {
      const mq = window.matchMedia("(prefers-color-scheme: light)");
      mq.addEventListener("change", () => {
        if (this.theme === "system") {
          this.#applyTheme();
        }
      });
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
}

export const settingsStore = new SettingsStore();
