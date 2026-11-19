import type { AlarmDto } from "./api";

export const ALARM_SNOOZE_MINUTES = 5;

export interface RepeatDayOption {
  label: string;
  short: string;
  bit: number;
}

/**
 * Urutan tampilan Minggu..Sabtu. Nilai `bit` tetap mengikuti bitmask Rust:
 * bit 0 = Senin sampai bit 6 = Minggu.
 */
export const REPEAT_DAY_OPTIONS: RepeatDayOption[] = [
  { label: "Sun", short: "Su", bit: 1 << 6 },
  { label: "Mon", short: "M", bit: 1 << 0 },
  { label: "Tue", short: "Tu", bit: 1 << 1 },
  { label: "Wed", short: "We", bit: 1 << 2 },
  { label: "Thu", short: "Th", bit: 1 << 3 },
  { label: "Fri", short: "Fr", bit: 1 << 4 },
  { label: "Sat", short: "Sa", bit: 1 << 5 },
];

const WEEKDAYS_MASK = 0b0001_1111;
const WEEKENDS_MASK = 0b0110_0000;
const ALL_DAYS_MASK = 0b0111_1111;

/** Memformat jam alarm sebagai "HH:MM" 24 jam. */
export function formatAlarmTime(hour: number, minute: number): string {
  const safeHour = Math.max(0, Math.min(23, Math.floor(hour)));
  const safeMinute = Math.max(0, Math.min(59, Math.floor(minute)));
  return `${String(safeHour).padStart(2, "0")}:${String(safeMinute).padStart(2, "0")}`;
}

/** Mengubah bitmask pengulangan menjadi label yang mudah dibaca. */
export function formatRepeatDays(mask: number): string {
  if (mask <= 0) return "Once";
  if (mask === ALL_DAYS_MASK) return "Every day";
  if (mask === WEEKDAYS_MASK) return "Weekdays";
  if (mask === WEEKENDS_MASK) return "Weekends";

  const selected = REPEAT_DAY_OPTIONS.filter((day) => (mask & day.bit) !== 0).map(
    (day) => day.label
  );
  return selected.length > 0 ? selected.join(", ") : "Once";
}

/** Menampilkan status jadwal alarm relatif terhadap sekarang. */
export function formatNextRing(alarm: AlarmDto, nowMs: number): string {
  if (!alarm.enabled) return "Off";
  if (alarm.next_ring_at_ms === null) return "Not scheduled";

  const diffMs = alarm.next_ring_at_ms - nowMs;
  const snoozed = alarm.snoozed_until_ms !== null;

  if (diffMs <= 0) {
    return snoozed ? "Snoozed · ringing now" : "Ringing now";
  }

  const relative = formatCountdown(diffMs);
  return snoozed ? `Snoozed · ${relative.toLowerCase()}` : relative;
}

function formatCountdown(diffMs: number): string {
  const totalMinutes = Math.max(1, Math.ceil(diffMs / 60_000));

  if (totalMinutes < 60) {
    return `In ${totalMinutes} min`;
  }

  const totalHours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (totalHours < 24) {
    return minutes === 0 ? `In ${totalHours}h` : `In ${totalHours}h ${minutes}m`;
  }

  const days = Math.floor(totalHours / 24);
  const hours = totalHours % 24;
  return hours === 0 ? `In ${days}d` : `In ${days}d ${hours}h`;
}

/**
 * Menghitung kemunculan berikutnya dari jadwal alarm untuk kebutuhan tampilan.
 * Dipakai saat alarm nonaktif agar kartu tetap bisa menampilkan hitungan waktu.
 */
function computeDisplayNextRingMs(
  hour: number,
  minute: number,
  repeatDays: number,
  nowMs: number
): number | null {
  const today = new Date(nowMs);
  for (let offset = 0; offset <= 7; offset += 1) {
    const candidate = new Date(today);
    candidate.setDate(candidate.getDate() + offset);
    candidate.setHours(hour, minute, 0, 0);
    if (candidate.getTime() <= nowMs) continue;
    if (repeatDays !== 0) {
      const mondayIndex = (candidate.getDay() + 6) % 7;
      if ((repeatDays & (1 << mondayIndex)) === 0) continue;
    }
    return candidate.getTime();
  }
  return null;
}

function formatLongCountdown(diffMs: number): string {
  const totalMinutes = Math.max(1, Math.ceil(diffMs / 60_000));

  if (totalMinutes < 60) {
    return `in ${totalMinutes} ${totalMinutes === 1 ? "minute" : "minutes"}`;
  }

  const totalHours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (totalHours < 24) {
    const hoursLabel = `${totalHours} ${totalHours === 1 ? "hour" : "hours"}`;
    if (minutes === 0) return `in ${hoursLabel}`;
    return `in ${hoursLabel}, ${minutes} ${minutes === 1 ? "minute" : "minutes"}`;
  }

  const days = Math.floor(totalHours / 24);
  const hours = totalHours % 24;
  const daysLabel = `${days} ${days === 1 ? "day" : "days"}`;
  if (hours === 0) return `in ${daysLabel}`;
  return `in ${daysLabel}, ${hours} ${hours === 1 ? "hour" : "hours"}`;
}

/** Label hitungan pada kartu alarm, tetap tampil saat alarm dinonaktifkan. */
export function formatAlarmCountdown(alarm: AlarmDto, nowMs: number): string {
  const snoozed = alarm.enabled && alarm.snoozed_until_ms !== null;
  const target =
    alarm.enabled && alarm.next_ring_at_ms !== null
      ? alarm.next_ring_at_ms
      : computeDisplayNextRingMs(alarm.hour, alarm.minute, alarm.repeat_days, nowMs);

  if (target === null) return "not scheduled";

  const diffMs = target - nowMs;
  if (diffMs <= 0) {
    return snoozed ? "snoozed · ringing now" : "ringing now";
  }

  const relative = formatLongCountdown(diffMs);
  return snoozed ? `snoozed · ${relative}` : relative;
}
