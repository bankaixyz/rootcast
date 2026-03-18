export function SiteFooter() {
  return (
    <footer className="site-footer">
      <div className="site-footer__inner">
        <div className="site-footer__left">
          <span className="site-footer__brand">World ID Replicator</span>
          <span className="site-footer__powered">Powered by Bankai</span>
        </div>
        <div className="site-footer__links">
          <a href="/dashboard" className="site-footer__link">
            Dashboard
          </a>
          <a
            href="https://sepolia.explorer.bankai.xyz/"
            className="site-footer__link"
            target="_blank"
            rel="noreferrer"
          >
            Explorer
          </a>
          <a
            href="https://docs.bankai.xyz/docs"
            className="site-footer__link"
            target="_blank"
            rel="noreferrer"
          >
            Docs
          </a>
          <a href="https://github.com/bankaixyz/world-id-replicator" className="site-footer__link" target="_blank" rel="noreferrer">
            GitHub
          </a>
          <a
            href="https://x.com/bankaihq"
            className="site-footer__link"
            target="_blank"
            rel="noreferrer"
          >
            Twitter
          </a>
        </div>
      </div>
    </footer>
  );
}
