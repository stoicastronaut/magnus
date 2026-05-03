import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./App.css";
import "./styles/tokens.css";
import "./styles/mascot.css";
import { installGlobalDiagnosticsHandlers } from "./services/tauri";

installGlobalDiagnosticsHandlers();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
