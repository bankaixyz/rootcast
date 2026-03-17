const TARGET_CHAIN_METADATA: Record<
  string,
  { addressBaseUrl: string; label: string; order: number; txBaseUrl: string }
> = {
  "arbitrum-sepolia": {
    addressBaseUrl: "https://sepolia.arbiscan.io/address/",
    label: "Arbitrum",
    order: 0,
    txBaseUrl: "https://sepolia.arbiscan.io/tx/",
  },
  "base-sepolia": {
    addressBaseUrl: "https://sepolia.basescan.org/address/",
    label: "Base",
    order: 1,
    txBaseUrl: "https://sepolia.basescan.org/tx/",
  },
  "op-sepolia": {
    addressBaseUrl: "https://sepolia-optimism.etherscan.io/address/",
    label: "OP",
    order: 2,
    txBaseUrl: "https://sepolia-optimism.etherscan.io/tx/",
  },
  "starknet-sepolia": {
    addressBaseUrl: "https://sepolia.voyager.online/contract",
    label: "Starknet",
    order: 3,
    txBaseUrl: "https://sepolia.voyager.online/tx/",
  },
};

export function chainLabel(chainName: string) {
  return TARGET_CHAIN_METADATA[chainName]?.label ?? chainName;
}

export function chainOrder(chainName: string) {
  return TARGET_CHAIN_METADATA[chainName]?.order ?? Number.MAX_SAFE_INTEGER;
}

export function allKnownTargetChains() {
  return Object.keys(TARGET_CHAIN_METADATA).sort(
    (left, right) => chainOrder(left) - chainOrder(right),
  );
}

export function chainTxUrl(chainName: string, hash: string) {
  const metadata = TARGET_CHAIN_METADATA[chainName];
  return metadata ? `${metadata.txBaseUrl}${hash}` : hash;
}

export function chainAddressUrl(chainName: string, address: string) {
  const metadata = TARGET_CHAIN_METADATA[chainName];
  return metadata ? `${metadata.addressBaseUrl}${address}` : address;
}

export function sourceTxUrl(hash: string) {
  return `https://sepolia.etherscan.io/tx/${hash}`;
}

export function bankaiBlockUrl(_blockNumber: number) {
  return "https://sepolia.dashboard.bankai.xyz";
}
