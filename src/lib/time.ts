import type { StopwatchDto, TimerDto } from "./api";

export interface FormattedTime {
  hours: string;
  minutes: string;
  seconds: string;
  tenths: string;
  /** Format gabungan: "HH:MM:SS.d" jika showTenths = true, atau "HH:MM:SS" jika false */
  formatted: string;
}

/**
 * Menghitung total milidetik yang telah berlalu berdasarkan snapshot DTO dan timestamp `nowMs`.
 * Mencegah nilai negatif jika jam sistem mengalami pergeseran kecil.
 */
export function computeElapsedMs(sw: StopwatchDto, nowMs: number): number {
  if (sw.started_at_ms !== null) {
    const delta = Math.max(0, nowMs - sw.started_at_ms);
    return Math.max(0, sw.accumulated_ms + delta);
  }
  return Math.max(0, sw.accumulated_ms);
}

/**
 * Memformat milidetik menjadi komponen jam, menit, detik, dan sepersepuluh detik.
 * Jika jam > 99, digit jam bertambah secara dinamis (tidak dipotong).
 */
export function formatElapsed(elapsedMs: number, showTenths = true): FormattedTime {
  const safeMs = Math.max(0, Math.floor(elapsedMs));
  const totalSeconds = Math.floor(safeMs / 1000);
  const tenthsDigit = Math.floor((safeMs % 1000) / 100);

  const hoursNum = Math.floor(totalSeconds / 3600);
  const minutesNum = Math.floor((totalSeconds % 3600) / 60);
  const secondsNum = totalSeconds % 60;

  const hours = String(hoursNum).padStart(2, "0");
  const minutes = String(minutesNum).padStart(2, "0");
  const seconds = String(secondsNum).padStart(2, "0");
  const tenths = String(tenthsDigit);

  const formatted = showTenths
    ? `${hours}:${minutes}:${seconds}.${tenths}`
    : `${hours}:${minutes}:${seconds}`;

  return { hours, minutes, seconds, tenths, formatted };
}

/**
 * Menghitung sisa waktu timer (dalam ms) berdasarkan snapshot DTO dan timestamp `nowMs`.
 */
export function computeRemainingMs(timer: TimerDto, nowMs: number): number {
  if (timer.started_at_ms !== null) {
    const elapsed = Math.max(0, nowMs - timer.started_at_ms);
    return Math.max(0, timer.remaining_ms - elapsed);
  }
  return Math.max(0, timer.remaining_ms);
}

/**
 * Memecah durasi milidetik menjadi jam, menit, detik (dibulatkan ke atas jika ada sisa ms saat countdown).
 */
export function splitDurationMs(durationMs: number): {
  hours: number;
  minutes: number;
  seconds: number;
} {
  const totalSeconds = Math.max(0, Math.ceil(durationMs / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  return { hours, minutes, seconds };
}

/**
 * Menggabungkan komponen jam, menit, detik menjadi milidetik.
 */
export function partsToMs(hours: number, minutes: number, seconds: number): number {
  const h = Math.max(0, Math.min(99, Math.floor(hours || 0)));
  const m = Math.max(0, Math.min(59, Math.floor(minutes || 0)));
  const s = Math.max(0, Math.min(59, Math.floor(seconds || 0)));
  return (h * 3600 + m * 60 + s) * 1000;
}
