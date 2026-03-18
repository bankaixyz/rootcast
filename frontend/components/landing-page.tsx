import type { RootSnapshot } from "@/lib/api";
import { CHAIN_LOGO_MAP, EthereumLogo } from "@/components/chain-logos";
import { ReplicationCard } from "@/components/replication-history-table";
import { Navbar } from "@/components/navbar";
import { SiteFooter } from "@/components/site-footer";
import type { ReactNode } from "react";

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

type FaqItem = {
  question: string;
  answer: ReactNode;
};

const FAQ_ITEMS: FaqItem[] = [
  {
    question: "What is World ID proof of personhood?",
    answer: (
      <p>
        World ID proof of personhood lets someone show they are a unique human,
        not a bot or a swarm of duplicate accounts. For applications, the
        important property is simple: one person counts once. In crypto, that
        is usually called Sybil resistance. It matters anywhere fairness
        depends on distinct humans, from governance and voting to social apps,
        rewards, and other coordination systems.
      </p>
    ),
  },
  {
    question: "Why do identity roots need to be replicated across chains?",
    answer: (
      <p>
        Under the hood, proof of personhood is represented as a Merkle tree,
        and users prove membership against its current root. If that root only
        lives on Ethereum, every application has to anchor back to Ethereum to
        trust it. Replicating the root across chains gives each supported chain
        the same canonical human set locally, so applications can verify the
        same World ID state wherever they run.
      </p>
    ),
  },
  {
    question: "How does the replication process work end-to-end?",
    answer: (
      <p>
        When World ID publishes a new root on Ethereum, we record the exact
        source block and wait for it to finalize. Then Bankai proves the
        relevant Ethereum storage value through its stateless light client
        architecture, and we verify that result inside SP1. The output is a
        proof of the exact L1-backed root, which we submit to destination
        registries so they can store the same trusted value locally.
      </p>
    ),
  },
  {
    question: "What is Bankai and how does it enable trustless proofs?",
    answer: (
      <p>
        Bankai is an interoperability system built around stateless light
        clients. Instead of trusting a relayer or keeping a destination-side
        light client synced forever, you verify a proof when you need the data.
        If the Bankai proof verifies, you know the committed chain data is
        valid according to consensus. That is what lets this system move World
        ID roots across chains without introducing a new trust layer.
      </p>
    ),
  },
  {
    question: "How quickly are roots replicated after an L1 update?",
    answer: (
      <p>
        We do not replicate the moment a new root appears, because the source
        Ethereum block has to finalize first. After that, proving and fan-out
        take about 90 seconds end to end. World ID root updates happen roughly
        once per hour, so the system is designed around correctness first and
        fast cross-chain availability second.
      </p>
    ),
  },
  {
    question: "Can I add support for my own chain?",
    answer: (
      <p>
        Usually, yes. Any destination that can verify the proof and store the
        resulting World ID root can join the replication set. On EVM chains,
        that often means reusing the same basic pattern: a Groth16 verifier
        plus a registry contract for verified roots, so new integrations can be
        very fast. On non-EVM chains, it depends on the available verifier
        environment and may require a small chain-specific contract or program
        to store the roots.
      </p>
    ),
  },
  {
    question: "How is this different from a traditional bridge?",
    answer: (
      <p>
        Traditional bridges usually ask you to trust a relayer, validator set,
        oracle network, or some other off-chain coordination layer. This system
        works differently: correctness comes from proving finalized chain data
        with light-client-style consensus verification. And because the light
        client is stateless, we do not need to keep destination-side clients
        continuously synced just to stay ready. That gives you a very different
        trust model and much lighter always-on destination infrastructure.
      </p>
    ),
  },
];

type LandingPageProps = {
  snapshot?: RootSnapshot | null;
};

export function LandingPage({ snapshot }: LandingPageProps) {
  return (
    <div className="landing">
      <Navbar currentPage="landing" />

      <section className="landing-hero">
        <div className="landing-hero__content">
          <span className="landing-hero__badge">Powered by Bankai</span>
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
          {FAQ_ITEMS.map(({ question, answer }) => (
            <details className="faq-item" key={question}>
              <summary className="faq-item__question">{question}</summary>
              <div className="faq-item__answer">
                {answer}
              </div>
            </details>
          ))}
        </div>
      </section>

      <section className="landing-cta">
        <span className="landing-cta__eyebrow">Open source</span>
        <h2 className="landing-cta__title">
          Bring proof of personhood to your chain
        </h2>
        <p className="landing-cta__sub">
          Deploy an identity root registry on any chain and join the replication
          network.
        </p>
        <a href="#" className="landing-btn landing-btn--primary">
          Get Started on GitHub
        </a>
      </section>

      <SiteFooter />
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

const VIZ = {
  size: 440,
  cx: 220,
  cy: 220,
  hubSize: 60,
  radius: 175,
  spreadDeg: 240,
  nodeSize: 40,
};

const VIZ_CHAINS = [
  "Solana","Monad", "HyperEVM", "Starknet", "Tempo", "MegaETH", "Plasma", "Base", "OP", "Arbitrum", 
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

      {nodes.map((node) => {
        const line = trimVizLine(node.x, node.y);

        return (
          <g key={node.name}>
            <line
              x1={line.x1} y1={line.y1} x2={line.x2} y2={line.y2}
              stroke="rgba(255,255,255,0.06)" strokeWidth="1"
            />
            <line
              x1={line.x1} y1={line.y1} x2={line.x2} y2={line.y2}
              className="hero-viz__signal"
              style={{ animationDelay: `${node.delay}s` }}
            />
            <foreignObject
              x={node.x - VIZ.nodeSize / 2} y={node.y - VIZ.nodeSize / 2}
              width={VIZ.nodeSize} height={VIZ.nodeSize}
              className="hero-viz__node"
            >
              <div className="hero-viz__node-inner">
                <ChainIcon name={node.name} />
              </div>
            </foreignObject>
          </g>
        );
      })}

      <foreignObject
        x={VIZ.cx - 30} y={VIZ.cy - 30}
        width={60} height={60}
        className="hero-viz__node"
      >
        <div className="hero-viz__hub-icon">
          <EthereumLogo size={42} />
        </div>
      </foreignObject>
    </svg>
  );
}

function ChainIcon({ name }: { name: string }) {
  const Logo = CHAIN_LOGO_MAP[name];
  if (!Logo) return null;
  return <Logo size={24} />;
}

function trimVizLine(x: number, y: number) {
  const dx = x - VIZ.cx;
  const dy = y - VIZ.cy;
  const distance = Math.hypot(dx, dy) || 1;
  const ux = dx / distance;
  const uy = dy / distance;
  const startInset = VIZ.hubSize / 2 + 4;
  const endInset = VIZ.nodeSize / 2 + 4;

  return {
    x1: VIZ.cx + ux * startInset,
    y1: VIZ.cy + uy * startInset,
    x2: x - ux * endInset,
    y2: y - uy * endInset,
  };
}
