import { Provider as JotaiProvider } from "jotai";
import React from "react";
import ReactDOM from "react-dom/client";

import App from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <JotaiProvider>
    <React.StrictMode>
      <App />
    </React.StrictMode>
  </JotaiProvider>,
);
