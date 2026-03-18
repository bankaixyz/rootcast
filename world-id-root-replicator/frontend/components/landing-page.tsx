import type { RootSnapshot } from "@/lib/api";
import { CHAIN_LOGO_MAP, EthereumLogo } from "@/components/chain-logos";
import { ReplicationCard } from "@/components/replication-history-table";

const STEPS = [
  {
    title: "Observe",
    description:
      "Monitor the World ID identity manager on Ethereum for new Merkle root updates.",
  },
  {
    title: "Finalize",
    description:
      "Wait for the source block to reach consensus finality on Ethereum L1.",
  },
  {
    title: "Prove",
    description:
      "Generate a zero-knowledge storage proof using Bankai\u2019s stateless light client.",
  },
  {
    title: "Replicate",
    description:
      "Submit the proven root to identity registries on every destination chain.",
  },
];

const FAQ_QUESTIONS = [
  "What is World ID proof of personhood?",
  "Why do identity roots need to be replicated across chains?",
  "How does the replication process work end-to-end?",
  "What is Bankai and how does it enable trustless proofs?",
  "Is the replication fully trustless?",
  "How quickly are roots replicated after an L1 update?",
  "Which blockchains are currently supported?",
  "Can I add support for my own chain?",
  "What happens if a replication attempt fails?",
  "How is this different from a traditional bridge?",
];

type LandingPageProps = {
  snapshot?: RootSnapshot | null;
};

export function LandingPage({ snapshot }: LandingPageProps) {
  return (
    <div className="landing">
      <nav className="landing-nav">
        <span className="landing-nav__brand">World ID Root Replicator</span>
        <div className="landing-nav__links">
          <a
            href="https://sepolia.dashboard.bankai.xyz"
            className="landing-nav__link"
            target="_blank"
            rel="noreferrer"
          >
            Bankai
          </a>
          <a href="#" className="landing-nav__link">
            GitHub
          </a>
        </div>
      </nav>

      <section className="landing-hero">
        <div className="landing-hero__content">
          <span className="landing-hero__eyebrow">Powered by Bankai</span>
          <h1 className="landing-hero__headline">
            Proof of personhood,
            <br />
            on every chain
          </h1>
          <p className="landing-hero__sub">
            Trustlessly replicate World ID identity roots from Ethereum across
            10+ blockchains using zero-knowledge proofs and stateless light
            client technology.
          </p>
          <div className="landing-hero__actions">
            <a href="/dashboard" className="landing-btn landing-btn--primary">
              View Live Dashboard
            </a>
            <a href="#" className="landing-btn landing-btn--ghost">
              View Source
            </a>
          </div>
        </div>
        <div className="landing-hero__visual">
          <ReplicationBurst />
        </div>
      </section>

      <section className="landing-stats">
        <div className="landing-stats__inner">
          <Stat value="10" label="Destination Chains" />
          <Stat value="ZK" label="Proven Roots" />
          <Stat value="0" label="Trust Assumptions" />
        </div>
      </section>

      <section className="landing-section">
        <span className="landing-section__eyebrow">How it works</span>
        <h2 className="landing-section__title">
          Four steps to universal proof of personhood
        </h2>
        <div className="landing-how">
          {STEPS.map((step, i) => (
            <article className="landing-step" key={step.title}>
              <span className="landing-step__number">
                {String(i + 1).padStart(2, "0")}
              </span>
              <h3 className="landing-step__title">{step.title}</h3>
              <p className="landing-step__desc">{step.description}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="landing-section">
        <span className="landing-section__eyebrow">Live status</span>
        <h2 className="landing-section__title">
          Replicated across the ecosystem
        </h2>
        {snapshot ? (
          <ReplicationCard root={snapshot} />
        ) : (
          <p className="landing-section__empty">
            No completed replications yet. The latest replication will appear
            here once roots have been relayed to destination chains.
          </p>
        )}
      </section>

      <section className="landing-section">
        <span className="landing-section__eyebrow">FAQ</span>
        <h2 className="landing-section__title">
          Frequently asked questions
        </h2>
        <div className="landing-faq">
          {FAQ_QUESTIONS.map((question) => (
            <details className="faq-item" key={question}>
              <summary className="faq-item__question">{question}</summary>
              <div className="faq-item__answer">
                <p>Coming soon.</p>
              </div>
            </details>
          ))}
        </div>
      </section>

      <footer className="landing-footer">
        <span className="landing-footer__eyebrow">Open source</span>
        <h2 className="landing-footer__title">
          Bring proof of personhood to your chain
        </h2>
        <p className="landing-footer__sub">
          Deploy an identity root registry on any EVM-compatible chain and join
          the replication network.
        </p>
        <a href="#" className="landing-btn landing-btn--primary">
          Get Started on GitHub
        </a>
      </footer>
    </div>
  );
}

function Stat({ value, label }: { value: string; label: string }) {
  return (
    <div className="landing-stat">
      <span className="landing-stat__value">{value}</span>
      <span className="landing-stat__label">{label}</span>
    </div>
  );
}

const VIZ = { size: 440, cx: 220, cy: 220, radius: 175, spreadDeg: 240 };

const VIZ_CHAINS = [
  "Base", "OP", "Arbitrum", "Starknet", "Solana",
  "Monad", "HyperEVM", "MegaETH", "Tempo", "Plasma",
];

function replicationNodes() {
  const start = -VIZ.spreadDeg / 2;
  const step = VIZ.spreadDeg / (VIZ_CHAINS.length - 1);
  return VIZ_CHAINS.map((name, i) => {
    const rad = ((start + i * step) * Math.PI) / 180;
    return {
      name,
      x: VIZ.cx + VIZ.radius * Math.cos(rad),
      y: VIZ.cy + VIZ.radius * Math.sin(rad),
      delay: i * 0.65,
    };
  });
}

function ReplicationBurst() {
  const nodes = replicationNodes();

  return (
    <svg
      aria-hidden="true"
      className="hero-viz"
      viewBox={`0 0 ${VIZ.size} ${VIZ.size}`}
    >
      <defs>
        <radialGradient id="hero-hub-glow">
          <stop offset="0%" stopColor="white" stopOpacity="0.14" />
          <stop offset="100%" stopColor="white" stopOpacity="0" />
        </radialGradient>
      </defs>

      <circle
        cx={VIZ.cx} cy={VIZ.cy} r={70}
        fill="none" stroke="rgba(255,255,255,0.035)" strokeWidth="1"
      />
      <circle
        cx={VIZ.cx} cy={VIZ.cy} r={120}
        fill="none" stroke="rgba(255,255,255,0.025)" strokeWidth="1"
      />

      <circle cx={VIZ.cx} cy={VIZ.cy} r={55} fill="url(#hero-hub-glow)" />

      {nodes.map((node) => (
        <g key={node.name}>
          <line
            x1={VIZ.cx} y1={VIZ.cy} x2={node.x} y2={node.y}
            stroke="rgba(255,255,255,0.06)" strokeWidth="1"
          />
          <line
            x1={VIZ.cx} y1={VIZ.cy} x2={node.x} y2={node.y}
            className="hero-viz__signal"
            style={{ animationDelay: `${node.delay}s` }}
          />
          <foreignObject
            x={node.x - 14} y={node.y - 14}
            width={28} height={28}
            className="hero-viz__node"
          >
            <div className="hero-viz__node-inner">
              <ChainIcon name={node.name} />
            </div>
          </foreignObject>
        </g>
      ))}

      <foreignObject
        x={VIZ.cx - 16} y={VIZ.cy - 16}
        width={32} height={32}
        className="hero-viz__node"
      >
        <div className="hero-viz__hub-icon">
          <EthereumLogo size={20} />
        </div>
      </foreignObject>
      <text
        x={VIZ.cx} y={VIZ.cy + 28}
        className="hero-viz__hub-label"
        textAnchor="middle"
      >
        ETHEREUM L1
      </text>
    </svg>
  );
}

function ChainIcon({ name }: { name: string }) {
  const Logo = CHAIN_LOGO_MAP[name];
  if (!Logo) return null;
  return <Logo size={16} />;
}
