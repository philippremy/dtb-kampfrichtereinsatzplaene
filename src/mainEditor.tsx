import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import Editor from "./Editor";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Editor />
  </React.StrictMode>
);
