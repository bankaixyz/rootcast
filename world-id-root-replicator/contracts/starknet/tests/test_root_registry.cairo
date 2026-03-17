use snforge_std::{declare, ContractClassTrait, DeclareResultTrait};
use starknet::ContractAddress;
use world_id_root_registry_starknet::{
    IWorldIdRootRegistryDispatcher, IWorldIdRootRegistryDispatcherTrait,
};

fn setup() -> (IWorldIdRootRegistryDispatcher, ContractAddress) {
    let contract = declare("WorldIdRootRegistry").unwrap().contract_class();

    let program_vk: u256 = 0x123456789abcdef_u256;
    let mut constructor_calldata = array![];
    Serde::serialize(@program_vk, ref constructor_calldata);

    let (contract_address, _) = contract.deploy(@constructor_calldata).unwrap();

    (
        IWorldIdRootRegistryDispatcher { contract_address },
        contract_address,
    )
}

#[test]
fn test_constructor_sets_program_vk() {
    let (registry, _) = setup();

    assert(registry.program_vk() == 0x123456789abcdef_u256, 'wrong program vk');
    assert(registry.latest_root() == 0_u256, 'wrong latest root');
    assert(registry.latest_source_block() == 0_u256, 'wrong latest block');
}

#[test]
fn test_exposes_verifier_class_hash() {
    let (registry, _) = setup();

    assert(
        registry.verifier_class_hash()
            == 0x79b72f62c1c6aad55c0ee0ecc68132a32db268306a19c451c35191080b7b611,
        'wrong verifier class hash',
    );
}
