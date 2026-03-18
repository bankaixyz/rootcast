use sp1_sdk::{include_elf, HashableKey, Prover, ProverClient};

const WORLD_ID_ROOT_REPLICATOR_ELF: &[u8] = include_elf!("world-id-root-replicator-program");

fn main() {
    let (_, vk) = ProverClient::builder()
        .cpu()
        .build()
        .setup(WORLD_ID_ROOT_REPLICATOR_ELF);
    println!("{}", vk.bytes32());
}
