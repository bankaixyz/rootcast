//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use alloy_primitives::{hex::FromHex, Address, FixedBytes, U256};
use bankai_sdk::{Bankai, HashingFunction, Network};
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use clap::Parser;
use sp1_sdk::network::NetworkMode;
use sp1_sdk::{include_elf, Prover, SP1Stdin};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
const WORLD_ID_ELF: &[u8] = include_elf!("world-id");
const SEPOLIA_IDENTITY_MANAGER: &str = "0xb2EaD588f14e69266d1b87936b75325181377076";
const LATEST_ROOT_SLOT: &str = "0x000000000000000000000000000000000000000000000000000000000000012e";

/// The arguments for the command.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    execute: bool,

    #[arg(long)]
    prove: bool,

    #[arg(long, default_value = "20")]
    n: u32,
}

#[tokio::main]
async fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();
    dotenv::dotenv().ok();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the bankai client.
    let exec_rpc = std::env::var("EXECUTION_RPC").ok();
    let bankai = Bankai::new(Network::Sepolia, exec_rpc, None, None);

    // Get the latest finalized L1 block from Bankai
    let latest_block = bankai
        .api
        .ethereum()
        .execution()
        .height(&BankaiBlockFilterDto::finalized())
        .await
        .unwrap();

    println!("got latest block: {}", latest_block.height);

    let contract = Address::from_hex(SEPOLIA_IDENTITY_MANAGER).unwrap();

    let key_bytes: FixedBytes<32> = FixedBytes::from_hex(LATEST_ROOT_SLOT).unwrap();
    let mpt_key = U256::from_be_bytes(key_bytes.into());

    let bundle = bankai
        .init_batch(None, HashingFunction::Keccak)
        .await
        .unwrap()
        .ethereum_storage_slot(latest_block.height, contract, vec![mpt_key])
        .execute()
        .await
        .unwrap();

    println!(
        "Got ProofBundle for world id root at block {}",
        latest_block.height
    );

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&bundle);

    if args.execute {
        // Setup mock prover for execute
        let client = sp1_sdk::ProverClient::builder().mock().build();

        // Execute the program
        let (output, report) = client.execute(WORLD_ID_ELF, &stdin).run().unwrap();
        println!("Program executed successfully.");
        println!("world id root: {output:?}");

        // Record the number of cycles executed.
        println!("Number of cycles: {}", report.total_instruction_count());
    } else {
        // Setup the program for proving.
        let private_key = std::env::var("NETWORK_PRIVATE_KEY").unwrap();

        let client = sp1_sdk::ProverClient::builder()
            .network_for(NetworkMode::Mainnet)
            .private_key(&private_key)
            .build();

        let (pk, vk) = client.setup(WORLD_ID_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, &stdin)
            .groth16()
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
