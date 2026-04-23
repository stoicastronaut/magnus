import { useState } from "react";
import { HomePage } from "./components/HomePage";
import { SettingsPage } from "./components/SettingsPage";

type View = "home" | "settings";
type Theme = "dark" | "light";

function App() {
  const [view, setView] = useState<View>("home");
  const [theme, setTheme] = useState<Theme>("dark");

  function toggleTheme() {
    setTheme((t) => (t === "dark" ? "light" : "dark"));
  }

  return (
    <div className="magnus-app" data-theme={theme} style={{ height: "100%" }}>
      <div style={{ display: view === "home" ? "contents" : "none" }}>
        <HomePage onSettings={() => setView("settings")} theme={theme} onToggleTheme={toggleTheme} />
      </div>
      {view === "settings" && <SettingsPage onBack={() => setView("home")} />}
    </div>
  );
}

export default App;
