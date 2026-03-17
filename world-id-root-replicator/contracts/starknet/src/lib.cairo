#[starknet::interface]
pub trait IWorldIdRootRegistry<TContractState> {
    fn submit_root(ref self: TContractState, proof: Array<felt252>);
    fn root_at(self: @TContractState, source_block_number: u256) -> u256;
    fn latest_root(self: @TContractState) -> u256;
    fn latest_source_block(self: @TContractState) -> u256;
    fn program_vk(self: @TContractState) -> u256;
    fn verifier_class_hash(self: @TContractState) -> felt252;
    fn verify_sp1_proof_view(self: @TContractState, proof: Array<felt252>) -> Option<(u256, Span<u256>)>;
}

#[starknet::contract]
pub mod WorldIdRootRegistry {
    use starknet::storage::{Map, StoragePointerReadAccess, StoragePointerWriteAccess};
    use starknet::syscalls::library_call_syscall;
    use starknet::SyscallResultTrait;

    /// Garaga's shared SP1 verifier class hash for BN254 Groth16 verification.
    const SP1_VERIFIER_CLASS_HASH: felt252 =
        0x79b72f62c1c6aad55c0ee0ecc68132a32db268306a19c451c35191080b7b611;

    #[storage]
    struct Storage {
        roots: Map<u256, u256>,
        latest_root: u256,
        latest_source_block: u256,
        program_vk: u256,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    pub enum Event {
        RootReplicated: RootReplicated,
        VerifierConfigured: VerifierConfigured,
    }

    #[derive(Drop, starknet::Event)]
    pub struct RootReplicated {
        #[key]
        pub source_block_number: u256,
        pub root: u256,
    }

    #[derive(Drop, starknet::Event)]
    pub struct VerifierConfigured {
        #[key]
        pub verifier_class_hash: felt252,
        pub program_vk: u256,
    }

    #[constructor]
    fn constructor(ref self: ContractState, program_vk: u256) {
        assert(program_vk != 0_u256, 'Invalid program vk');
        self.program_vk.write(program_vk);
        self.latest_root.write(0_u256);
        self.latest_source_block.write(0_u256);
        self.emit(
            VerifierConfigured {
                verifier_class_hash: SP1_VERIFIER_CLASS_HASH,
                program_vk,
            },
        );
    }

    #[abi(embed_v0)]
    impl WorldIdRootRegistryImpl of super::IWorldIdRootRegistry<ContractState> {
        fn submit_root(ref self: ContractState, proof: Array<felt252>) {
            let (program_vk, public_inputs) = self._verify_sp1_proof(proof);
            assert(program_vk == self.program_vk.read(), 'Wrong program');
            assert(public_inputs.len() >= 2, 'Invalid public inputs');

            let source_block_number = *public_inputs.at(0);
            let root = *public_inputs.at(1);

            assert(source_block_number > self.latest_source_block.read(), 'Stale source block');

            let existing_root = self.roots.entry(source_block_number).read();
            assert(
                existing_root == 0_u256 || existing_root == root,
                'Conflicting root',
            );

            self.roots.entry(source_block_number).write(root);
            self.latest_source_block.write(source_block_number);
            self.latest_root.write(root);
            self.emit(RootReplicated { source_block_number, root });
        }

        fn root_at(self: @ContractState, source_block_number: u256) -> u256 {
            self.roots.entry(source_block_number).read()
        }

        fn latest_root(self: @ContractState) -> u256 {
            self.latest_root.read()
        }

        fn latest_source_block(self: @ContractState) -> u256 {
            self.latest_source_block.read()
        }

        fn program_vk(self: @ContractState) -> u256 {
            self.program_vk.read()
        }

        fn verifier_class_hash(self: @ContractState) -> felt252 {
            SP1_VERIFIER_CLASS_HASH
        }

        fn verify_sp1_proof_view(
            self: @ContractState,
            proof: Array<felt252>,
        ) -> Option<(u256, Span<u256>)> {
            let (_, public_inputs) = self._verify_sp1_proof(proof);
            Option::Some((self.program_vk.read(), public_inputs))
        }
    }

    #[generate_trait]
    impl InternalFunctions of InternalFunctionsTrait {
        fn _verify_sp1_proof(
            self: @ContractState,
            proof: Array<felt252>,
        ) -> (u256, Span<u256>) {
            let mut result_serialized = library_call_syscall(
                SP1_VERIFIER_CLASS_HASH.try_into().unwrap(),
                selector!("verify_sp1_groth16_proof_bn254"),
                proof.span(),
            )
                .unwrap_syscall();

            let result = Serde::<Option<(u256, Span<u256>)>>::deserialize(ref result_serialized)
                .unwrap();
            assert(result.is_some(), 'Invalid proof');
            result.unwrap()
        }
    }
}
