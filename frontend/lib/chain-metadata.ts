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
  "solana-devnet": {
    addressUrl: (address) => `https://solscan.io/account/${address}?cluster=devnet`,
    label: "Solana",
    order: 0,
    targetLabel: "Program",
    txUrl: (hash) => `https://solscan.io/tx/${hash}?cluster=devnet`,
  },
  chiado: {
    addressUrl: (address) => `https://gnosis-chiado.blockscout.com/address/${address}`,
    label: "Chiado",
    order: 1,
    targetLabel: "Registry",
    txUrl: (hash) => `https://gnosis-chiado.blockscout.com/tx/${hash}`,
  },
  "monad-testnet": {
    addressUrl: (address) => `https://testnet.monadscan.com/address/${address}`,
    label: "Monad",
    order: 2,
    targetLabel: "Registry",
    txUrl: (hash) => `https://testnet.monadscan.com/tx/${hash}`,
  },
  "tempo-testnet": {
    addressUrl: (address) => `https://explore.testnet.tempo.xyz/address/${address}`,
    label: "Tempo",
    order: 3,
    targetLabel: "Registry",
    txUrl: (hash) => `https://explore.testnet.tempo.xyz/tx/${hash}`,
  },
  "starknet-sepolia": {
    addressUrl: (address) => `https://sepolia.voyager.online/contract/${address}`,
    label: "Starknet",
    order: 4,
    targetLabel: "Contract",
    txUrl: (hash) => `https://sepolia.voyager.online/tx/${hash}`,
  },
  "megaeth-testnet": {
    addressUrl: (address) => `https://testnet-mega.etherscan.io/address/${address}`,
    label: "MegaETH",
    order: 5,
    targetLabel: "Registry",
    txUrl: (hash) => `https://testnet-mega.etherscan.io/tx/${hash}`,
  },
  "plasma-testnet": {
    addressUrl: (address) => `https://testnet.plasmascan.to/address/${address}`,
    label: "Plasma",
    order: 6,
    targetLabel: "Registry",
    txUrl: (hash) => `https://testnet.plasmascan.to/tx/${hash}`,
  },
  "hyperevm-testnet": {
    addressUrl: (address) => `https://testnet.purrsec.com/address/${address}`,
    label: "HyperEVM",
    order: 7,
    targetLabel: "Registry",
    txUrl: (hash) => `https://testnet.purrsec.com/tx/${hash}`,
  },
  "op-sepolia": {
    addressUrl: (address) => `https://sepolia-optimism.etherscan.io/address/${address}`,
    label: "OP",
    order: 8,
    targetLabel: "Registry",
    txUrl: (hash) => `https://sepolia-optimism.etherscan.io/tx/${hash}`,
  },
  "base-sepolia": {
    addressUrl: (address) => `https://sepolia.basescan.org/address/${address}`,
    label: "Base",
    order: 9,
    targetLabel: "Registry",
    txUrl: (hash) => `https://sepolia.basescan.org/tx/${hash}`,
  },
  "arbitrum-sepolia": {
    addressUrl: (address) => `https://sepolia.arbiscan.io/address/${address}`,
    label: "Arbitrum",
    order: 10,
    targetLabel: "Registry",
    txUrl: (hash) => `https://sepolia.arbiscan.io/tx/${hash}`,
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
