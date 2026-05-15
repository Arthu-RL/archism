import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";

export default function App() {
  const [step, setStep] = useState(1);
  const [disks, setDisks] = useState<string[]>([]);
  const [status, setStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const [config, setConfig] = useState({
    disk: "",
    hostname: "archism",
    username: "",
    locale: "en_US.UTF-8",
    timezone: "America/Sao_Paulo",
    keymap: "br-abnt2",
    dm: "gdm",
    gpu: "nvidia",
  });

  useEffect(() => {
    invoke<string[]>("get_disks").then(setDisks).catch(console.error);
  }, []);

  const handleInstall = async () => {
    setIsLoading(true);
    setStatus("Partitioning and installing base system... This will take a while.");
    try {
      const response = await invoke<string>("start_installation", { config });
      setStatus(`Success: ${response}`);
      setStep(3);
    } catch (error) {
      setStatus(`Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-neutral-900 text-neutral-100 font-sans flex items-center justify-center p-6">
      <div className="max-w-2xl w-full bg-neutral-800 rounded-2xl shadow-2xl overflow-hidden border border-neutral-700">
        
        {/* Header */}
        <div className="bg-neutral-950 p-6 border-b border-neutral-700 flex justify-between items-center">
          <div>
            <h1 className="text-2xl font-bold tracking-tight text-white">Archism</h1>
            <p className="text-neutral-400 text-sm">One-shot Arch Linux Setup</p>
          </div>
          <div className="flex gap-2">
            <span className={`h-2 w-8 rounded-full ${step >= 1 ? "bg-blue-500" : "bg-neutral-700"}`} />
            <span className={`h-2 w-8 rounded-full ${step >= 2 ? "bg-blue-500" : "bg-neutral-700"}`} />
            <span className={`h-2 w-8 rounded-full ${step >= 3 ? "bg-blue-500" : "bg-neutral-700"}`} />
          </div>
        </div>

        {/* Body */}
        <div className="p-8">
          {step === 1 && (
            <div className="space-y-6 animate-fade-in">
              <h2 className="text-xl font-semibold mb-4">System Configuration</h2>
              
              <div className="grid grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className="text-sm font-medium text-neutral-300">Target Disk (Will be ERASED)</label>
                  <select 
                    className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-3 text-white outline-none focus:border-blue-500 transition-colors"
                    value={config.disk}
                    onChange={(e) => setConfig({ ...config, disk: e.target.value })}
                  >
                    <option value="">Select a disk...</option>
                    {disks.map(d => <option key={d} value={d}>{d}</option>)}
                  </select>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-neutral-300">Hostname</label>
                  <input 
                    type="text"
                    className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-3 text-white outline-none focus:border-blue-500 transition-colors"
                    value={config.hostname}
                    onChange={(e) => setConfig({ ...config, hostname: e.target.value })}
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-neutral-300">Username</label>
                  <input 
                    type="text"
                    className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-3 text-white outline-none focus:border-blue-500 transition-colors"
                    value={config.username}
                    onChange={(e) => setConfig({ ...config, username: e.target.value })}
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium text-neutral-300">Display Manager</label>
                  <select 
                    className="w-full bg-neutral-950 border border-neutral-700 rounded-lg p-3 text-white outline-none focus:border-blue-500 transition-colors"
                    value={config.dm}
                    onChange={(e) => setConfig({ ...config, dm: e.target.value })}
                  >
                    <option value="gdm">GDM (GNOME)</option>
                    <option value="sddm">SDDM (KDE Plasma)</option>
                    <option value="lightdm">LightDM</option>
                  </select>
                </div>
              </div>

              <div className="mt-8 flex justify-end">
                <button 
                  disabled={!config.disk || !config.username}
                  onClick={() => setStep(2)}
                  className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white px-6 py-3 rounded-lg font-medium transition-colors"
                >
                  Review & Continue
                </button>
              </div>
            </div>
          )}

          {step === 2 && (
            <div className="space-y-6 animate-fade-in">
              <h2 className="text-xl font-semibold mb-2">Review Summary</h2>
              <div className="bg-neutral-950 p-4 rounded-lg border border-neutral-700 space-y-2 font-mono text-sm">
                <p><span className="text-neutral-500">Disk:</span> <span className="text-red-400">{config.disk} (DATA WILL BE LOST)</span></p>
                <p><span className="text-neutral-500">Hostname:</span> {config.hostname}</p>
                <p><span className="text-neutral-500">User:</span> {config.username}</p>
                <p><span className="text-neutral-500">Environment:</span> {config.dm}</p>
              </div>
              
              {status && <p className="text-sm text-blue-400 mt-4">{status}</p>}

              <div className="mt-8 flex justify-between">
                <button 
                  onClick={() => setStep(1)}
                  disabled={isLoading}
                  className="text-neutral-400 hover:text-white px-4 py-2 transition-colors"
                >
                  Back
                </button>
                <button 
                  onClick={handleInstall}
                  disabled={isLoading}
                  className="bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white px-6 py-3 rounded-lg font-medium transition-colors flex items-center gap-2"
                >
                  {isLoading ? "Installing..." : "Wipe Disk & Install Arch"}
                </button>
              </div>
            </div>
          )}

          {step === 3 && (
            <div className="text-center py-12 animate-fade-in">
              <div className="w-16 h-16 bg-green-500/20 text-green-500 rounded-full flex items-center justify-center mx-auto mb-6 text-3xl">✓</div>
              <h2 className="text-2xl font-bold mb-2">Installation Complete</h2>
              <p className="text-neutral-400 mb-8">Arch Linux has been successfully configured on {config.disk}.</p>
              <button 
                onClick={() => invoke('run_cmd', { cmd: 'reboot', args: [] })}
                className="bg-neutral-100 hover:bg-white text-neutral-900 px-8 py-3 rounded-lg font-bold transition-colors"
              >
                Reboot System
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}