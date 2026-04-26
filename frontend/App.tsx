import { useState } from "react";
import { HomePage } from "./components/HomePage";
import { SettingsPage } from "./components/SettingsPage";

type View = "home" | "settings";
type Theme = "dark" | "light";

function App() {
  const [view, setView] = useState<View>("home");
  const [theme, setTheme] = useState<Theme>("dark");
  const [settingsVersion, setSettingsVersion] = useState(0);

  function toggleTheme() {
    setTheme((t) => (t === "dark" ? "light" : "dark"));
  }

  return (
    <div className="magnus-app" data-theme={theme} style={{ height: "100%" }}>
      <div style={{ display: view === "home" ? "contents" : "none" }}>
        <HomePage
          onSettings={() => setView("settings")}
          theme={theme}
          onToggleTheme={toggleTheme}
          settingsVersion={settingsVersion}
        />
      </div>
      {view === "settings" && (
        <SettingsPage
          onBack={() => {
            setSettingsVersion((v) => v + 1);
            setView("home");
          }}
          theme={theme}
          onThemeChange={setTheme}
        />
      )}
    </div>
  );
}

export default App;
