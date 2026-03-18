import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";
import { PublicKey, SystemProgram } from "@solana/web3.js";

describe("world-id-root-registry-solana", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.WorldIdRootRegistrySolana as anchor.Program;

  it("initializes the registry state PDA", async () => {
    const [statePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId,
    );
    const programVkey = Array(32).fill(7);

    await program.methods
      .initialize(programVkey)
      .accounts({
        payer: provider.wallet.publicKey,
        state: statePda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const state = await program.account.registryState.fetch(statePda);
    assert.deepEqual(state.programVkeyHash, programVkey);
    assert.equal(state.latestSourceBlock.toNumber(), 0);
  });
});
