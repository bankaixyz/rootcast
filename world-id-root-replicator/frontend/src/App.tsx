import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Home } from "./pages/Home";
import { Dashboard } from "./pages/Dashboard";

export function App() {
  return (
    <BrowserRouter>
      <div className="site-background">
        <div className="site-background__mesh site-background__mesh--top" />
        <div className="site-background__mesh site-background__mesh--bottom" />
        <div className="site-background__grid" />
      </div>
      <div className="site-shell">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/dashboard" element={<Dashboard />} />
        </Routes>
      </div>
    </BrowserRouter>
  );
}
