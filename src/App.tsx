import { useState } from "react";
import { HomePage } from "./components/HomePage";
import { SettingsPage } from "./components/SettingsPage";

type View = "home" | "settings";

function App() {
  const [view, setView] = useState<View>("home");

  if (view === "settings") {
    return <SettingsPage onBack={() => setView("home")} />;
  }
  return <HomePage onSettings={() => setView("settings")} />;
}

export default App;
