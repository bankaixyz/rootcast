type NavbarProps = {
  currentPage?: "landing" | "dashboard";
};

export function Navbar({ currentPage = "landing" }: NavbarProps) {
  return (
    <nav className="landing-nav">
      <a href="/" className="landing-nav__brand">
        
      </a>
      <div className="landing-nav__links">
        {currentPage === "dashboard" && (
          <a href="/" className="landing-nav__link">
            Home
          </a>
        )}
        {currentPage === "landing" && (
          <a href="/dashboard" className="landing-nav__link">
            Dashboard
          </a>
        )}
        <a
          href="https://bankai.xyz"
          className="landing-nav__link"
          target="_blank"
          rel="noreferrer"
        >
          Bankai
        </a>
        <a href="https://github.com/bankaixyz/world-id-replicator" className="landing-nav__link" target="_blank" rel="noreferrer">
          GitHub
        </a>
      </div>
    </nav>
  );
}
