import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { version } from "../package.json";

type LogLine = {
  text: string;
  kind: "info" | "ok" | "error";
}

type ProgressEvent = {
  stage: string;
  percent: number;
}

type InstallConfig = {
  disk: string;
  hostname: string;
  username: string;
  locale: string;
  timezone: string;
  keymap: string;
  dm: string;
  gpu: string;
  swap_size: number;
}

const LOCALES = [
  "en_US.UTF-8", "pt_BR.UTF-8", "de_DE.UTF-8",
  "fr_FR.UTF-8", "es_ES.UTF-8", "it_IT.UTF-8",
];

const TIMEZONES = [
  "America/Sao_Paulo", "America/New_York", "America/Los_Angeles",
  "America/Chicago", "Europe/London", "Europe/Berlin",
  "Europe/Paris", "Asia/Tokyo", "Asia/Shanghai", "Australia/Sydney",
];

const KEYMAPS = [
  "br-abnt2", "us", "de-latin1", "fr-pc", "es", "it", "uk"
];

const STEPS = ["Configurar", "Revisar", "Instalar", "Sucesso"] as const;

const inputCls =
  "w-full bg-neutral-950 border border-neutral-700 rounded-xl px-4 py-3 " +
  "text-sm text-white outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all shadow-inner";

const selectCls =
  "w-full bg-neutral-950 border border-neutral-700 rounded-xl px-4 py-3 " +
  "text-sm text-white outline-none focus:border-blue-500 focus:ring-2 focus:ring-blue-500/20 transition-all shadow-inner " +
  "cursor-pointer text-left text-white";

const labelCls = "block text-xs font-semibold text-neutral-400 mb-2 tracking-wide uppercase";

export default function App() {
  const [step, setStep] = useState<0 | 1 | 2 | 3>(0);
  const [disks, setDisks] = useState<string[]>([]);
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [progress, setProgress] = useState<ProgressEvent>({ stage: "", percent: 0 });
  const [error, setError] = useState("");
  const logRef = useRef<HTMLDivElement>(null);

  const [config, setConfig] = useState<InstallConfig>({
    disk: "",
    hostname: "archbox",
    username: "",
    locale: "en_US.UTF-8",
    timezone: "America/Sao_Paulo",
    keymap: "br-abnt2",
    dm: "gdm",
    gpu: "none",
    swap_size: 8,
  });

  useEffect(() => {
    invoke<string[]>("get_disks").then(setDisks).catch(console.error);

    const unlistenLog = listen<LogLine>("installer:log", (e) =>
      setLogs((prev) => [...prev, e.payload])
    );
    const unlistenProg = listen<ProgressEvent>("installer:progress", (e) =>
      setProgress(e.payload)
    );

    return () => {
      unlistenLog.then((f) => f());
      unlistenProg.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [logs]);

  const cfg = (k: keyof InstallConfig, v: string | number) =>
    setConfig((c) => ({ ...c, [k]: v }));

  const handleInstall = async () => {
    setStep(2);
    setError("");
    setLogs([]);
    setProgress({ stage: "Preparando ambiente...", percent: 0 });
    try {
      await invoke("start_installation", { config });
      setStep(3);
    } catch (e) {
      setError(String(e));
    }
  };

  const canContinue =
    config.disk !== "" &&
    config.username.trim().length >= 2 &&
    config.hostname.trim().length >= 1 &&
    config.keymap !== "" &&
    config.swap_size >= 0;

  return (
    <div className="min-h-screen bg-neutral-950 text-neutral-100 flex items-center justify-center p-4 sm:p-6 md:p-8 antialiased">
      <div className="w-full max-w-2xl bg-neutral-900 border border-neutral-800 rounded-2xl shadow-2xl overflow-hidden transition-all duration-300">

        {/* ── Header ── */}
        <div className="p-6 border-b border-neutral-800 bg-neutral-900/50 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h1 className="text-xl font-black tracking-tight bg-linear-to-r from-blue-400 to-cyan-400 bg-clip-text text-transparent">⬡ Archism</h1>
            <p className="text-xs text-neutral-400 mt-0.5">Instalador Moderno do Arch Linux</p>
          </div>

          <div className="flex items-center gap-2">
            {STEPS.map((name, i) => (
              <div key={i} className="flex items-center">
                <div
                  className={`h-2 rounded-full transition-all duration-500 ${
                    i < step ? "bg-blue-500 w-6" : i === step ? "bg-cyan-400 w-10" : "bg-neutral-800 w-4"
                  }`}
                  title={name}
                />
              </div>
            ))}
          </div>
        </div>

        {/* ── Main Content ── */}
        <div className="p-6 sm:p-8">

          {/* Step 0: Configuration */}
          {step === 0 && (
            <div className="space-y-6">
              <div className="border-b border-neutral-800 pb-2">
                <h2 className="text-base font-bold text-neutral-200">Base System Configuration</h2>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-5">
                
                {/* Target Disk */}
                <div className="sm:col-span-2 space-y-1">
                  <label className={labelCls}>
                    Target Disk
                    <span className="text-red-400 font-medium normal-case block sm:inline sm:ml-2">
                      ⚠️ ATTENTION: All data will be destroyed!
                    </span>
                  </label>
                  <select
                    style={{ colorScheme: "dark" }}
                    className={selectCls}
                    value={config.disk}
                    onChange={(e) => cfg("disk", e.target.value)}
                  >
                    <option value="">Select an available drive...</option>
                    {disks.map((d) => (
                      <option key={d} value={d}>{d}</option>
                    ))}
                  </select>
                </div>

                <div className="space-y-1">
                  <label className={labelCls}>Machine Name (Hostname)</label>
                  <input
                    type="text"
                    className={inputCls}
                    value={config.hostname}
                    onChange={(e) => cfg("hostname", e.target.value)}
                  />
                </div>

                <div className="space-y-1">
                  <label className={labelCls}>Username</label>
                  <input
                    type="text"
                    className={inputCls}
                    placeholder="ex: arthur"
                    value={config.username}
                    onChange={(e) =>
                      cfg("username", e.target.value.toLowerCase().replace(/[^a-z0-9_-]/g, ""))
                    }
                  />
                </div>

                <div className="space-y-1">
                  <label className={labelCls}>Location (Locale)</label>
                  <select 
                    style={{ colorScheme: "dark" }}
                    className={selectCls} 
                    value={config.locale} 
                    onChange={(e) => cfg("locale", e.target.value)}
                  >
                    {LOCALES.map((l) => (
                      <option key={l} value={l}>{l}</option>
                    ))}
                  </select>
                </div>

                <div className="space-y-1">
                  <label className={labelCls}>Timezone</label>
                  <select 
                    style={{ colorScheme: "dark" }}
                    className={selectCls} 
                    value={config.timezone} 
                    onChange={(e) => cfg("timezone", e.target.value)}
                  >
                    {TIMEZONES.map((t) => (
                      <option key={t} value={t}>{t}</option>
                    ))}
                  </select>
                </div>

                <div className="space-y-1">
                  <label className={labelCls}>Keyboard Layout (Keymap)</label>
                  <select 
                    style={{ colorScheme: "dark" }}
                    className={selectCls} 
                    value={config.keymap} 
                    onChange={(e) => cfg("keymap", e.target.value)}
                  >
                    {KEYMAPS.map((k) => (
                      <option key={k} value={k}>{k}</option>
                    ))}
                  </select>
                </div>

                <div className="space-y-1">
                  <label className={labelCls}>Swap Memory (In Gigabytes)</label>
                  <div className="relative flex items-center">
                    <input
                      type="number"
                      min="0"
                      max="64"
                      className={inputCls}
                      value={config.swap_size}
                      onChange={(e) => cfg("swap_size", Math.max(0, parseInt(e.target.value) || 0))}
                    />
                    <span className="absolute right-10 text-sm font-bold text-neutral-500 pointer-events-none">GB</span>
                  </div>
                </div>

                <div className="space-y-1">
                  <label className={labelCls}>Graphics Environment</label>
                  <select 
                    style={{ colorScheme: "dark" }}
                    className={selectCls} 
                    value={config.dm} 
                    onChange={(e) => cfg("dm", e.target.value)}
                  >
                    <option value="gdm">GNOME (GDM)</option>
                    <option value="sddm">KDE Plasma (SDDM)</option>
                    <option value="lightdm">XFCE (LightDM)</option>
                  </select>
                </div>

                <div className="sm:col-span-2 space-y-1">
                  <label className={labelCls}>Video Driver (GPU)</label>
                  <select 
                    style={{ colorScheme: "dark" }}
                    className={selectCls} 
                    value={config.gpu} 
                    onChange={(e) => cfg("gpu", e.target.value)}
                  >
                    <option value="none">None / Generic Virtual Machine (Generic Mesa)</option>
                    <option value="nvidia">NVIDIA (Stable Proprietary)</option>
                    <option value="amd">AMD (Mesa + Vulkan OpenSource)</option>
                    <option value="intel">Intel (Mesa + Media Driver)</option>
                  </select>
                </div>
              </div>

              <div className="mt-8 flex justify-end">
                <button
                  disabled={!canContinue}
                  onClick={() => setStep(1)}
                  className="w-full sm:w-auto cursor-pointer bg-blue-600 hover:bg-blue-500 disabled:opacity-30 disabled:cursor-not-allowed
                             text-white text-sm font-bold px-6 py-3 rounded-xl shadow-lg shadow-blue-600/10 transition-all duration-200"
                >
                  Next to Review →
                </button>
              </div>
            </div>
          )}

          {/* Step 1: Review */}
          {step === 1 && (
            <div className="space-y-6">
              <div className="border-b border-neutral-800 pb-2">
                <h2 className="text-base font-bold text-neutral-200">Confirm Destruction and Write</h2>
              </div>

              <div className="bg-neutral-950 rounded-xl border border-neutral-800 p-5 font-mono text-xs space-y-3 shadow-inner">
                {(
                  [
                    ["Target Disk", <span className="text-red-400 font-bold">{config.disk} (WILL BE FORMATTED AND CLEANED)</span>],
                    ["Hostname",   config.hostname],
                    ["Username",   config.username],
                    ["Locale",     config.locale],
                    ["Timezone",   config.timezone],
                    ["Keymap",     config.keymap],
                    ["Swap File",  `${config.swap_size} GB`],  
                    ["Interface",  config.dm.toUpperCase()],
                    ["Driver GPU", config.gpu.toUpperCase()],
                    ["Partitions",  "GPT · EFI (2GB FAT32) · ROOT (Restante ext4)"],
                  ] as [string, React.ReactNode][]
                ).map(([k, v]) => (
                  <div key={k} className="flex flex-col sm:flex-row sm:gap-4 border-b border-neutral-900 pb-1 last:border-0">
                    <span className="text-neutral-500 w-28 shrink-0 font-bold uppercase tracking-wider text-[10px]">{k}</span>
                    <span className="text-neutral-200 break-all">{v}</span>
                  </div>
                ))}
              </div>

              <div className="flex flex-col sm:flex-row justify-between items-center gap-4 pt-2">
                <button
                  onClick={() => setStep(0)}
                  className="w-full sm:w-auto cursor-pointer text-sm text-neutral-400 hover:text-white px-4 py-3 rounded-xl border border-neutral-800 hover:bg-neutral-800/40 transition-all"
                >
                  ← Back to Modify Options
                </button>
                <button
                  onClick={handleInstall}
                  className="w-full sm:w-auto cursor-pointer bg-red-600 hover:bg-red-500 text-white text-sm font-bold
                             px-6 py-3 rounded-xl shadow-lg shadow-red-600/20 transition-all duration-200"
                >
                  Delete Everything & Install Arch
                </button>
              </div>
            </div>
          )}

          {/* Step 2: Installing */}
          {step === 2 && (
            <div className="space-y-5">
              <div className="flex items-center justify-between">
                <h2 className="text-sm font-bold text-neutral-200">
                  {error ? "Installation Aborted due to error" : "Installation Progress"}
                </h2>
                <span className="text-xs font-mono font-bold bg-neutral-950 px-2 py-1 rounded border border-neutral-800 text-cyan-400">
                  {progress.percent}%
                </span>
              </div>

              <div className="h-2 bg-neutral-950 rounded-full overflow-hidden border border-neutral-800">
                <div
                  className={`h-full rounded-full transition-all duration-500 ${
                    error ? "bg-red-500" : "bg-linear-to-r from-blue-500 to-cyan-500"
                  }`}
                  style={{ width: `${progress.percent}%` }}
                />
              </div>
              <p className="text-xs text-neutral-400 italic h-4 animate-pulse">
                {progress.stage}
              </p>

              <div
                ref={logRef}
                className="bg-neutral-950 rounded-xl border border-neutral-800 p-4
                           font-mono text-[11px] h-64 overflow-y-auto shadow-inner leading-relaxed"
              >
                {logs.length === 0 ? (
                  <span className="text-neutral-600 flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-neutral-700 animate-ping" />
                    Waiting for Arch Linux pipelines...
                  </span>
                ) : (
                  logs.map((l, i) => (
                    <div
                      key={i}
                      className={
                        l.kind === "ok"
                          ? "text-green-400"
                          : l.kind === "error"
                          ? "text-red-400 font-bold"
                          : "text-neutral-300"
                      }
                    >
                      {l.text}
                    </div>
                  ))
                )}
              </div>

              {error && (
                <div className="bg-red-950/50 border border-red-800/60 rounded-xl p-4 text-xs text-red-300 space-y-2">
                  <p className="font-bold uppercase tracking-wider text-red-400">Fatal Error Detected</p>
                  <p className="font-mono bg-neutral-950/60 p-2 rounded border border-red-900/30 overflow-x-auto">{error}</p>
                  <button
                    onClick={() => setStep(0)}
                    className="inline-block cursor-pointer text-red-400 hover:text-red-200 font-bold underline transition-colors pt-1"
                  >
                    ← Back to start and try again
                  </button>
                </div>
              )}
            </div>
          )}

          {/* Step 3: Finished */}
          {step === 3 && (
            <div className="py-8 text-center space-y-5">
              <div className="w-16 h-16 bg-green-500/10 text-green-400 border border-green-500/20 rounded-full flex items-center justify-center mx-auto text-2xl shadow-xl shadow-green-500/5">
                ✓
              </div>
              <div className="space-y-1">
                <h2 className="text-lg font-black tracking-tight text-white">Installation Successfully Completed!</h2>
                <p className="text-sm text-neutral-400">
                  O Arch Linux foi implantado no dispositivo <span className="font-mono text-neutral-200 bg-neutral-950 px-1.5 py-0.5 rounded border border-neutral-800">{config.disk}</span>.
                </p>
              </div>
              <button
                onClick={() => invoke("reboot_system")}
                className="w-full sm:w-auto cursor-pointer bg-white hover:bg-neutral-200 text-neutral-900 text-sm
                           font-bold px-8 py-3 rounded-xl shadow-xl transition-all duration-200"
              >
                Restart Computer
              </button>
            </div>
          )}
        </div>
      </div>

      <p className="fixed bottom-4 text-center text-[10px] uppercase font-bold tracking-widest text-neutral-600 pointer-events-none">
        Archism Installer · Powered by Tauri & Rust · v{version} · © 2026 ArthurRL · All rights reserved
      </p>
    </div>
  );
}