import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import BugReporter from "./BugReporter";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <BugReporter />
    </React.StrictMode>
);