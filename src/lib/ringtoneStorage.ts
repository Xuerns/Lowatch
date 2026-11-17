export const MAX_RINGTONE_BYTES = 10 * 1024 * 1024; // 10 MB
export const MAX_RINGTONE_DURATION_SEC = 30; // 30 seconds

const ALLOWED_EXTENSIONS = new Set([".mp3", ".wav", ".ogg", ".m4a"]);
const ALLOWED_MIME_TYPES = new Set([
  "audio/mpeg",
  "audio/mp3",
  "audio/wav",
  "audio/wave",
  "audio/x-wav",
  "audio/ogg",
  "application/ogg",
  "audio/mp4",
  "audio/x-m4a",
  "audio/aac",
]);

const DB_NAME = "lowatch-media";
const DB_VERSION = 1;
const STORE_NAME = "ringtones";
const RINGTONE_KEY = "custom-timer-ringtone";

export interface CustomRingtoneRecord {
  name: string;
  type: string;
  size: number;
  durationSec: number;
  blob: Blob;
}

function getExtension(filename: string): string {
  const dot = filename.lastIndexOf(".");
  if (dot === -1) return "";
  return filename.slice(dot).toLowerCase();
}

export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

/**
 * Memvalidasi tipe file, ukuran maksimum (10 MB), dan membaca durasi audio.
 * Mengembalikan `needsTrim: true` apabila durasi > 30 detik.
 */
export async function validateAndInspectAudioFile(
  file: File
): Promise<{ durationSec: number; needsTrim: boolean }> {
  const ext = getExtension(file.name);
  const mime = file.type.toLowerCase();

  const extAllowed = ALLOWED_EXTENSIONS.has(ext);
  const mimeAllowed = mime === "" || ALLOWED_MIME_TYPES.has(mime);

  if (!extAllowed || !mimeAllowed) {
    throw new Error("Unsupported file type. Please choose an MP3, WAV, OGG, or M4A file.");
  }

  if (file.size <= 0) {
    throw new Error("Selected audio file is empty.");
  }

  if (file.size > MAX_RINGTONE_BYTES) {
    throw new Error(
      `File is too large (${formatFileSize(file.size)}). Maximum allowed size is 10 MB.`
    );
  }

  const durationSec = await readAudioDuration(file);
  if (!Number.isFinite(durationSec) || durationSec <= 0) {
    throw new Error("Could not read audio duration. The file may be corrupted.");
  }

  return {
    durationSec,
    needsTrim: durationSec > MAX_RINGTONE_DURATION_SEC,
  };
}

/**
 * Mengekstrak array puncak amplitudo ternormalisasi (0..1) dari file audio untuk visualisasi waveform.
 */
export async function extractWaveformPeaks(
  file: File,
  barCount = 96
): Promise<number[]> {
  const AudioContextCtor =
    window.AudioContext ??
    (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!AudioContextCtor) {
    return Array.from({ length: barCount }, () => 0.4);
  }

  const ctx = new AudioContextCtor();
  try {
    const arrayBuffer = await file.arrayBuffer();
    const decoded = await ctx.decodeAudioData(arrayBuffer);
    const channelData = decoded.getChannelData(0);
    const totalSamples = channelData.length;
    const blockSize = Math.max(1, Math.floor(totalSamples / barCount));
    const peaks: number[] = [];

    let maxPeak = 0;
    for (let i = 0; i < barCount; i++) {
      const start = i * blockSize;
      const end = Math.min(totalSamples, start + blockSize);
      // Sample with stride so long songs decode & summarize in <10ms
      const stride = Math.max(1, Math.floor((end - start) / 64));
      let peak = 0;
      for (let j = start; j < end; j += stride) {
        const abs = Math.abs(channelData[j] ?? 0);
        if (abs > peak) peak = abs;
      }
      if (peak > maxPeak) maxPeak = peak;
      peaks.push(peak);
    }

    const norm = maxPeak > 0 ? maxPeak : 1;
    return peaks.map((p) => Math.max(0.12, p / norm));
  } catch {
    return Array.from({ length: barCount }, () => 0.4);
  } finally {
    await ctx.close();
  }
}

/**
 * Memotong file audio pada rentang [startSec, endSec] menggunakan Web Audio API
 * dan mengembalikannya sebagai File WAV 16-bit PCM.
 */
export async function trimAudioFileToWav(
  file: File,
  startSec: number,
  endSec: number
): Promise<{ trimmedFile: File; durationSec: number }> {
  const AudioContextCtor =
    window.AudioContext ??
    (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!AudioContextCtor) {
    throw new Error("Web Audio API is not available to trim audio.");
  }

  const ctx = new AudioContextCtor();
  try {
    const arrayBuffer = await file.arrayBuffer();
    const decoded = await ctx.decodeAudioData(arrayBuffer);

    const safeStart = Math.max(0, Math.min(startSec, decoded.duration));
    const safeEnd = Math.max(safeStart + 0.5, Math.min(endSec, decoded.duration));
    const clippedEnd = Math.min(safeEnd, safeStart + MAX_RINGTONE_DURATION_SEC);

    const sampleRate = decoded.sampleRate;
    const startFrame = Math.floor(safeStart * sampleRate);
    const endFrame = Math.min(decoded.length, Math.floor(clippedEnd * sampleRate));
    const frameCount = Math.max(1, endFrame - startFrame);
    const numChannels = Math.min(2, decoded.numberOfChannels);

    const wavBuffer = encodePcmWav(decoded, startFrame, frameCount, numChannels);
    const baseName = file.name.replace(/\.[^.]+$/, "");
    const trimmedFile = new File([wavBuffer], `${baseName} (trimmed).wav`, {
      type: "audio/wav",
    });

    return {
      trimmedFile,
      durationSec: frameCount / sampleRate,
    };
  } finally {
    await ctx.close();
  }
}

function encodePcmWav(
  audioBuffer: AudioBuffer,
  startFrame: number,
  frameCount: number,
  numChannels: number
): ArrayBuffer {
  const bytesPerSample = 2;
  const dataSize = frameCount * numChannels * bytesPerSample;
  const buffer = new ArrayBuffer(44 + dataSize);
  const view = new DataView(buffer);

  writeAscii(view, 0, "RIFF");
  view.setUint32(4, 36 + dataSize, true);
  writeAscii(view, 8, "WAVE");
  writeAscii(view, 12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true); // PCM
  view.setUint16(22, numChannels, true);
  view.setUint32(24, audioBuffer.sampleRate, true);
  view.setUint32(28, audioBuffer.sampleRate * numChannels * bytesPerSample, true);
  view.setUint16(32, numChannels * bytesPerSample, true);
  view.setUint16(34, 16, true);
  writeAscii(view, 36, "data");
  view.setUint32(40, dataSize, true);

  const channels: Float32Array[] = [];
  for (let ch = 0; ch < numChannels; ch++) {
    channels.push(audioBuffer.getChannelData(ch));
  }

  let offset = 44;
  for (let i = 0; i < frameCount; i++) {
    const frameIdx = startFrame + i;
    for (let ch = 0; ch < numChannels; ch++) {
      const sample = Math.max(-1, Math.min(1, channels[ch][frameIdx] ?? 0));
      view.setInt16(offset, sample < 0 ? sample * 0x8000 : sample * 0x7fff, true);
      offset += 2;
    }
  }

  return buffer;
}

function writeAscii(view: DataView, offset: number, text: string): void {
  for (let i = 0; i < text.length; i++) {
    view.setUint8(offset + i, text.charCodeAt(i));
  }
}

function readAudioDuration(file: File): Promise<number> {
  return new Promise((resolve, reject) => {
    const objectUrl = URL.createObjectURL(file);
    const audio = new Audio();
    let settled = false;

    const cleanup = () => {
      audio.removeEventListener("loadedmetadata", onLoaded);
      audio.removeEventListener("error", onError);
      audio.src = "";
      URL.revokeObjectURL(objectUrl);
    };

    const timeoutId = setTimeout(() => {
      if (settled) return;
      settled = true;
      cleanup();
      reject(new Error("Timed out while reading audio file metadata."));
    }, 5000);

    const onLoaded = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timeoutId);
      const duration = audio.duration;
      cleanup();
      resolve(duration);
    };

    const onError = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timeoutId);
      cleanup();
      reject(new Error("This audio file cannot be decoded by the player."));
    };

    audio.preload = "metadata";
    audio.addEventListener("loadedmetadata", onLoaded);
    audio.addEventListener("error", onError);
    audio.src = objectUrl;
  });
}

function openMediaDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    if (typeof indexedDB === "undefined") {
      reject(new Error("IndexedDB is not available in this environment."));
      return;
    }
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME);
      }
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error ?? new Error("Failed to open local media storage."));
  });
}

export async function saveCustomRingtone(
  file: File,
  durationSec: number
): Promise<CustomRingtoneRecord> {
  const db = await openMediaDb();
  const record: CustomRingtoneRecord = {
    name: file.name,
    type: file.type,
    size: file.size,
    durationSec,
    blob: file,
  };

  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    store.put(record, RINGTONE_KEY);
    tx.oncomplete = () => {
      db.close();
      resolve(record);
    };
    tx.onerror = () => {
      db.close();
      reject(tx.error ?? new Error("Failed to save custom ringtone."));
    };
  });
}

export async function loadCustomRingtone(): Promise<CustomRingtoneRecord | null> {
  const db = await openMediaDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);
    const req = store.get(RINGTONE_KEY);
    req.onsuccess = () => {
      db.close();
      const val = req.result as CustomRingtoneRecord | undefined;
      resolve(val ?? null);
    };
    req.onerror = () => {
      db.close();
      reject(req.error ?? new Error("Failed to load custom ringtone."));
    };
  });
}

export async function clearCustomRingtone(): Promise<void> {
  const db = await openMediaDb();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    store.delete(RINGTONE_KEY);
    tx.oncomplete = () => {
      db.close();
      resolve();
    };
    tx.onerror = () => {
      db.close();
      reject(tx.error ?? new Error("Failed to clear custom ringtone."));
    };
  });
}
