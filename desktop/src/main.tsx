import React from "react";
import ReactDOM from "react-dom/client";
import "@fontsource/bricolage-grotesque/latin-500.css";
import "@fontsource/bricolage-grotesque/latin-600.css";
import "@fontsource/ibm-plex-sans/latin-400.css";
import "@fontsource/ibm-plex-sans/latin-500.css";
import "@fontsource/ibm-plex-sans/latin-600.css";
import "@fontsource/jetbrains-mono/latin-400.css";
import "../../tokens.css";
import "./styles.css";
import "./i18n";
import { App } from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
