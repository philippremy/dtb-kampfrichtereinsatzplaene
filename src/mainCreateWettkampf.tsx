import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import CreateWettkampf from "./CreateWettkampf";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <CreateWettkampf />
  </React.StrictMode>
);
