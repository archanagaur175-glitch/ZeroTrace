import { useState } from "react";
import { IconRail, type Screen } from "./components/shell/IconRail";
import { StatusBar } from "./components/shell/StatusBar";
import { RadarScreen } from "./components/radar/RadarScreen";
import { TelemetryScreen } from "./components/telemetry/TelemetryScreen";
import { StartupScreen } from "./components/startup/StartupScreen";
import { TrustScreen } from "./components/trust/TrustScreen";
import { SettingsScreen } from "./components/settings/SettingsScreen";

export default function App() {
  const [screen, setScreen] = useState<Screen>("radar");

  return (
    <div className="flex h-full w-full bg-base-950">
      <IconRail active={screen} onNavigate={setScreen} />
      <main className="flex-1 overflow-hidden">
        {screen === "radar" && <RadarScreen />}
        {screen === "telemetry" && <TelemetryScreen />}
        {screen === "startup" && <StartupScreen />}
        {screen === "trust" && <TrustScreen />}
        {screen === "settings" && <SettingsScreen />}
      </main>
      <StatusBar />
    </div>
  );
}
