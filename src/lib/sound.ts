let audioCtx: AudioContext | null = null;
let activeAudio: HTMLAudioElement | null = null;
let activeAudioUrl: string | null = null;
let alarmChimeInterval: ReturnType<typeof setInterval> | null = null;

/**
 * Menghentikan pemutaran nada dering custom yang sedang berjalan.
 */
export function stopCustomRingtone(): void {
  if (activeAudio) {
    activeAudio.pause();
    activeAudio.currentTime = 0;
    activeAudio.src = "";
    activeAudio = null;
  }
  if (activeAudioUrl) {
    URL.revokeObjectURL(activeAudioUrl);
    activeAudioUrl = null;
  }
}

/**
 * Menghentikan loop chime alarm bawaan.
 */
export function stopAlarmChimeLoop(): void {
  if (alarmChimeInterval !== null) {
    clearInterval(alarmChimeInterval);
    alarmChimeInterval = null;
  }
}

/**
 * Memutar chime alarm secara berulang sampai dihentikan.
 */
export function startAlarmChimeLoop(): void {
  stopAlarmChimeLoop();
  playTimerChime();
  alarmChimeInterval = setInterval(playTimerChime, 2_800);
}

/**
 * Menghentikan seluruh sumber suara alarm (chime maupun ringtone custom).
 */
export function stopAlarmSound(): void {
  stopAlarmChimeLoop();
  stopCustomRingtone();
}

/**
 * Memutar rentang waktu tertentu [startSec, endSec] dari file audio untuk preview trim.
 */
export function playAudioRangePreview(
  blob: Blob,
  startSec: number,
  endSec: number,
  onEnded?: () => void
): boolean {
  try {
    stopCustomRingtone();
    const url = URL.createObjectURL(blob);
    const audio = new Audio(url);
    activeAudio = audio;
    activeAudioUrl = url;

    const handleFinish = () => {
      audio.removeEventListener("timeupdate", onTimeUpdate);
      if (activeAudio === audio) {
        stopCustomRingtone();
      } else {
        URL.revokeObjectURL(url);
      }
      onEnded?.();
    };

    const onTimeUpdate = () => {
      if (audio.currentTime >= endSec) {
        handleFinish();
      }
    };

    audio.addEventListener("timeupdate", onTimeUpdate);
    audio.addEventListener("ended", handleFinish, { once: true });
    audio.addEventListener("error", handleFinish, { once: true });
    audio.addEventListener(
      "loadedmetadata",
      () => {
        audio.currentTime = Math.max(0, startSec);
        void audio.play().catch(handleFinish);
      },
      { once: true }
    );

    return true;
  } catch {
    stopCustomRingtone();
    onEnded?.();
    return false;
  }
}

/**
 * Memutar file audio custom dari Blob. Mengembalikan true jika berhasil memulai pemutaran.
 */
export function playCustomRingtone(
  blob: Blob,
  onEnded?: () => void,
  loop = false
): boolean {
  try {
    stopCustomRingtone();
    const url = URL.createObjectURL(blob);
    const audio = new Audio(url);
    audio.loop = loop;
    activeAudio = audio;
    activeAudioUrl = url;

    const handleFinish = () => {
      if (activeAudio === audio) {
        stopCustomRingtone();
      } else {
        URL.revokeObjectURL(url);
      }
      onEnded?.();
    };

    const handleFailure = () => {
      handleFinish();
      if (loop) {
        startAlarmChimeLoop();
      } else {
        playTimerChime();
      }
    };

    audio.addEventListener("ended", handleFinish, { once: true });
    audio.addEventListener("error", handleFailure, { once: true });

    void audio.play().catch(handleFailure);
    return true;
  } catch {
    stopCustomRingtone();
    if (loop) {
      startAlarmChimeLoop();
    } else {
      playTimerChime();
    }
    onEnded?.();
    return false;
  }
}

/**
 * Memainkan nada notifikasi singkat menggunakan Web Audio API saat timer selesai.
 */
export function playTimerChime(): void {
  try {
    const AudioContextCtor =
      window.AudioContext ??
      (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AudioContextCtor) return;

    if (!audioCtx) {
      audioCtx = new AudioContextCtor();
    }
    if (audioCtx.state === "suspended") {
      void audioCtx.resume();
    }

    const now = audioCtx.currentTime;
    const notes = [880, 1174.66];

    notes.forEach((freq, idx) => {
      if (!audioCtx) return;
      const osc = audioCtx.createOscillator();
      const gain = audioCtx.createGain();

      osc.type = "sine";
      osc.frequency.setValueAtTime(freq, now + idx * 0.16);

      gain.gain.setValueAtTime(0.001, now + idx * 0.16);
      gain.gain.exponentialRampToValueAtTime(0.14, now + idx * 0.16 + 0.02);
      gain.gain.exponentialRampToValueAtTime(0.001, now + idx * 0.16 + 0.35);

      osc.connect(gain);
      gain.connect(audioCtx.destination);

      osc.start(now + idx * 0.16);
      osc.stop(now + idx * 0.16 + 0.36);
    });
  } catch {
    // Ignore audio errors when Web Audio is unavailable or blocked
  }
}
