const TARGET_CHAIN_METADATA: Record<
  string,
  {
    addressUrl: (address: string) => string;
    label: string;
    order: number;
    targetLabel: string;
    txUrl: (hash: string) => string;
  }
> = {
  "arbitrum-sepolia": {
    addressUrl: (address) => `https://sepolia.arbiscan.io/address/${address}`,
    label: "Arbitrum",
    order: 0,
    targetLabel: "Registry",
    txUrl: (hash) => `https://sepolia.arbiscan.io/tx/${hash}`,
  },
  "base-sepolia": {
    addressUrl: (address) => `https://sepolia.basescan.org/address/${address}`,
    label: "Base",
    order: 1,
    targetLabel: "Registry",
    txUrl: (hash) => `https://sepolia.basescan.org/tx/${hash}`,
  },
  "op-sepolia": {
    addressUrl: (address) => `https://sepolia-optimism.etherscan.io/address/${address}`,
    label: "OP",
    order: 2,
    targetLabel: "Registry",
    txUrl: (hash) => `https://sepolia-optimism.etherscan.io/tx/${hash}`,
  },
  "starknet-sepolia": {
    addressUrl: (address) => `https://sepolia.voyager.online/contract/${address}`,
    label: "Starknet",
    order: 3,
    targetLabel: "Contract",
    txUrl: (hash) => `https://sepolia.voyager.online/tx/${hash}`,
  },
  "solana-devnet": {
    addressUrl: (address) => `https://solscan.io/account/${address}?cluster=devnet`,
    label: "Solana",
    order: 4,
    targetLabel: "Program",
    txUrl: (hash) => `https://solscan.io/tx/${hash}?cluster=devnet`,
  },
  chiado: {
    addressBaseUrl: "https://gnosis-chiado.blockscout.com/address/",
    label: "Chiado",
    order: 3,
    txBaseUrl: "https://gnosis-chiado.blockscout.com/tx/",
  },
  "monad-testnet": {
    addressBaseUrl: "https://testnet.monadscan.com/address/",
    label: "Monad",
    order: 4,
    txBaseUrl: "https://testnet.monadscan.com/tx/",
  },
  "hyperevm-testnet": {
    addressBaseUrl: "https://testnet.purrsec.com/address/",
    label: "HyperEVM",
    order: 5,
    txBaseUrl: "https://testnet.purrsec.com/tx/",
  },
  "tempo-testnet": {
    addressBaseUrl: "https://explore.tempo.xyz/address/",
    label: "Tempo",
    order: 6,
    txBaseUrl: "https://explore.tempo.xyz/tx/",
  },
  "megaeth-testnet": {
    addressBaseUrl: "https://testnet-mega.etherscan.io/address/",
    label: "MegaETH",
    order: 7,
    txBaseUrl: "https://testnet-mega.etherscan.io/tx/",
  },
  "plasma-testnet": {
    addressBaseUrl: "https://testnet.plasmascan.to/address/",
    label: "Plasma",
    order: 8,
    txBaseUrl: "https://testnet.plasmascan.to/tx/",
  },
};

export function chainLabel(chainName: string) {
  return TARGET_CHAIN_METADATA[chainName]?.label ?? chainName;
}

export function chainOrder(chainName: string) {
  return TARGET_CHAIN_METADATA[chainName]?.order ?? Number.MAX_SAFE_INTEGER;
}

export function chainTargetLabel(chainName: string) {
  return TARGET_CHAIN_METADATA[chainName]?.targetLabel ?? "Target";
}

export function allKnownTargetChains() {
  return Object.keys(TARGET_CHAIN_METADATA).sort(
    (left, right) => chainOrder(left) - chainOrder(right),
  );
}

export function chainTxUrl(chainName: string, hash: string) {
  const metadata = TARGET_CHAIN_METADATA[chainName];
  return metadata ? metadata.txUrl(hash) : hash;
}

export function chainAddressUrl(chainName: string, address: string) {
  const metadata = TARGET_CHAIN_METADATA[chainName];
  return metadata ? metadata.addressUrl(address) : address;
}

export function sourceTxUrl(hash: string) {
  return `https://sepolia.etherscan.io/tx/${hash}`;
}

export function bankaiBlockUrl(_blockNumber: number) {
  return "https://sepolia.dashboard.bankai.xyz";
}
