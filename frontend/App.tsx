import { useState } from "react";
import { HomePage } from "./components/HomePage";
import { SettingsPage } from "./components/SettingsPage";

type View = "home" | "settings";

function App() {
  const [view, setView] = useState<View>("home");

  return (
    <>
      <div style={{ display: view === "home" ? "contents" : "none" }}>
        <HomePage onSettings={() => setView("settings")} />
      </div>
      {view === "settings" && <SettingsPage onBack={() => setView("home")} />}
    </>
  );
}

export default App;
