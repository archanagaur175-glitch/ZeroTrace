import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ErrorBoundary } from "./components/shell/ErrorBoundary";
import "./styles/tokens.css";

window.addEventListener("error", (e) => {
  console.error("[zerotrace] uncaught", e.error ?? e.message);
});
window.addEventListener("unhandledrejection", (e) => {
  console.error("[zerotrace] unhandled rejection", e.reason);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);