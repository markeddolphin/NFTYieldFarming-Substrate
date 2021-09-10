# Smart contract Storage rent

## Storage rent : What is it ?

 Official doc : https://docs.rs/frame-support/3.0.0/frame_support/storage/child/enum.ChildInfo.html
 
According to the doc :
_An account of a contract instance is charged proportionally to the amount of storage its account uses_

As the smart contract is deployed in the contract pallet, it utilizes a certain amount of the contract pallet storage. Every contrat is paying a fee as it grows in storage size.

The endowment is the initial balance that is send to the contract (by the instanciator account). If this balance is lower than the fees that the contract need to pay, the contract will be set a "tombstone" and its storage will be cleaned up.
According to the doc _A tombstone contract can be restored by providing the data that was cleaned up when it became a tombstone as well as any additional funds needed to keep the contract alive_ -> any contract can be restored from the "tombstone" state.

## Storage rent: How it works ?

This fee can be simplified by :
`(Price of Storage - Endowment) * Rent Fraction`
_can be found in pallet contracts rent.rs:100 https://github.com/paritytech/substrate/blob/0ce623a5530aa029ac287cec121ccde327103490/frame/contracts/src/rent.rs#L100_

- Endowment: the initial balance send to the contract by the instanciator account. It then become the balance of the contract. At every block if their is a fee to pey it will be deducted to endowment. _The balance to transfer from the `origin` to the newly created contract._
- Rent Fraction: the fraction of the deposit that should be used as rent per block (by default is set as the number of blocks produced in one month)
- Price of Storage: the total price of the storage (detailed below)

`if price of storage < Endowment` then the rent for the block will be exempt
`if price of storage > Endowment` then the fee applies and is due per the current block

This fee is payed via the Endowment so at every block the endowment should pay the rent fee:
(n as block number, 0 as the contract instantation)
endowmentn  = endowment0 - rent0 - rent1 - .. - rentn-1 - rentn

**Note! :** By default the full endowment is allowed to pay the rent. so **endowment == allowance**. But is is possible manually set the allowance to dedicate less funds fo rent storage fees.
When the allowance is not enough to pay the fee then the contract goes to Tombstone (and the contract storage will be cleaned). Of course the endowment can be bailed out at any time and the storage restored.

A good way to manage your contract is to have enough Endowment to not pay fee per block. This endowment is never gone because it can be withdraw with the deletion of the contract. 
But if you let your storage grows and not bail out you endowment then you will pay fee per block until their is no allowance funds left and your contract will be set as tombstone state.

This mecanism ensure that their is economical incentive to keep usefull contract working and that other contract (that has not their endowment as high enough) will be removed after a certain amount of time.

### Price of storage: How is it Computed ?

Price of storage can be determinated by :
`Price = (Base Deposit + Storage Size + Contract Code Size) * Byte Price + (Items in Storage * Price per Item)`

Legend: **Name** (_variable name in contract pallet_): Description . _doc from contract pallet_
- **Base Deposit** (_DepositPerContract_): The base price of storage. This value is set by Default in the runtime. _Costs for additional storage are added to this base cost. This is a simple way to ensure that contracts with empty storage eventually get deleted by  making them pay rent. This creates an incentive to remove them early in order to save rent._
- **Storage Size** (_contract.storage_size_): length of storage expressed in bytes (total number of bytes used by the storage)
- **Contract Code Size** (_code_size_share_): length in bytes of the contract deployed 
- **Byte Price** (_DepositPerStorageByte_): Price of the storage per byte. This value is set by Default in the runtime. _The balance a contract needs to deposit per storage byte to stay alive indefinitely.  Let's suppose the deposit is 1,000 BU (balance units)/byte and the rent is 1 BU/byte/day, then a contract with 1,000,000 BU that uses 1,000 bytes of storage would pay no rent.  But if the balance reduced to 500,000 BU and the storage stayed the same at 1,000, then it would pay 500 BU/day._
- **Items in Storage** (_code_size_share_): Sum of each key-value pair stored by this contract.
- **Price per Item** (_DepositPerStorageItem_): Price for a key-value pair in storage. his value is set by Default in the runtime. _The balance a contract needs to deposit per storage item to stay alive indefinitely.  It works the same as [`Self::DepositPerStorageByte`] but for storage items._

### Storage rent: How to make it free ?

A solution to make the deactivate the storage rent process and make any contract live indefinitely is to set **Byte Price** & **Price per Item** to zero in your runtime. This way any storage added by the contract will not be charged.

### Storage rent: Example Case

_Code for this test & its runtime values can be found at the end_

Keep the variable to set by default:
- Base deposit: 80_000
- Byte Price: 10_000
- Price per Item: 10_000
- Rent Fraction: 400_000
```rust
	pub const DepositPerContract: u64 = 8 * DepositPerStorageByte::get();
	pub const DepositPerStorageByte: u64 = 10_000;
	pub const DepositPerStorageItem: u64 = 10_000;
	pub RentFraction: Perbill = Perbill::from_rational(4u32, 10_000u32);
```
source: https://github.com/paritytech/substrate/blob/0ce623a5530aa029ac287cec121ccde327103490/frame/contracts/src/tests.rs#L249

Lets deploy contract with these following values:
- Endowment: 30_000
- Contract Code Size: 1163 (bytes)
- Allowance: 30_000

**Rent0**: Initial
`fee = 400 000 * ((8 + 4 + 1163) * 10 000 + 10 000 - 30 000) = 4696U`
- Allowance: 30 000 - 4692 = 25304
- Account balance 5endowment left): 30 000 - 4692 = 25304

**rent5**: Call to add 4 bytes in storage and on intem key pair & advance of 5 blocks
`fee = (400 000 * ((8 + 8 + 1163) * 10 000 + 20 000 - 25 304) * 4 blocks))= 18856U`

- Allowance: 30 000 - 4692 - 18856 = 6448
- Account balance: 30 000 - 4692 - 18856 = 6448

**rent6**: Advance of 1 blocks
`fee = (400 000 * ((8 + 8 + 1163) * 10 000 + 20 000 - 6448) * 1 block)) = 4722U`

- Allowance: 30 000 - 4692 - 18856 - 4722 = 1730
- Account balance: 30 000 - 4692 - 18856 - 4722 = 1730

**Block7** : not enough funds allowance to pay the rent : the contract goes to tombstone

Code of the test below: Exemple Case

### How to restore contract from Tombstone state

When your contract is a tombstone their is no way to make a call to it. Their is a way to restore this contract.

**note!** if there there was only one instance of your contract on chain and it became a tombstone, their is no way to restore it. When there is several contracts with same code base (the code produced the same hash) they actually share it (the code is not duplicated on chain). As long as their is at least one contract with this code base (hash) on chain, it is possible to restore it. Otherwise all code is deleted.
see: https://github.com/paritytech/substrate/blob/282d57c0745b530fe7a9ebaffcd6ac36c09d0554/frame/contracts/src/tests.rs#L2671

In order to restore a contract form tombstone state you need :
- Instanciate via another account (not the one that instanciate the tombstone contract) the restauration code (to be detailled).
- Perform a call to the restauration contract with the account that instanciated the tombstone contract.

The restoration should be successfull !

Code of the test Provided below

### Source Code
1. Example Case:
```rust
#[test]
fn storage_rent_grows_whith_storage() {
	let (wasm, code_hash) = compile_module::<Test>("set_rent").unwrap();
	let endowment: BalanceOf<Test> = BalanceOf::<Test>::from(30_000u32);
	let allowance: BalanceOf<Test> = BalanceOf::<Test>::from(30_000u32);

	// Storage size
	ExtBuilder::default()
		.existential_deposit(50)
		.build()
		.execute_with(|| {
			// Create
			let _ = Balances::deposit_creating(&ALICE, 100_000_000u64);
			assert_ok!(Contracts::instantiate_with_code(
				Origin::signed(ALICE),
				endowment,
				GAS_LIMIT,
				wasm,
				// rent_allowance
				allowance.encode(),
				vec![],
			));
			let addr = Contracts::contract_address(&ALICE, &code_hash, &[]);
			let contract = ContractInfoOf::<Test>::get(&addr).unwrap().get_alive().unwrap();
			let code_len: BalanceOf<Test> =
				PrefabWasmModule::<Test>::from_storage_noinstr(contract.code_hash)
					.unwrap()
					.occupied_storage()
					.into();

			// The instantiation deducted the rent for one block immediately
			let rentFraction = <Test as Config>::RentFraction::get();
			// (base_deposit(8) + bytes in storage(4) + size of code) * byte_price + 1 storage item (10_000)
			let gross_rent_price = (8 + 4 + code_len) * 10_000 + 10_000;
			// - free_balance
			let net_rent_price = gross_rent_price.saturating_sub(endowment);
			let rent0 = rentFraction.mul_ceil(net_rent_price)
				// blocks to rent
				* 1;
			assert!(rent0 > 0);
			assert_eq!(contract.rent_allowance, allowance - rent0);
			assert_eq!(contract.deduct_block, 1);
			assert_eq!(Balances::free_balance(&addr), endowment - rent0);

			assert_ok!(Contracts::call(
				Origin::signed(ALICE),
				addr.clone(),
				0,
				GAS_LIMIT,
				call::set_storage_4_byte()
			));
			let contract = ContractInfoOf::<Test>::get(&addr)
				.unwrap()
				.get_alive()
				.unwrap();
			assert_eq!(
				contract.storage_size,
				4 + 4
			);
			assert_eq!(
				contract.pair_count,
				2,
			);

			// Advance 4 blocks
			initialize_block(5);

			// Trigger rent through call
			assert_ok!(
				Contracts::call(Origin::signed(ALICE), addr.clone(), 0, GAS_LIMIT, call::null())
			);

			// Check result
			let rent = <Test as Config>::RentFraction::get() // rent: 1300
				.mul_ceil(((8 + 8 + code_len) * 10_000 + 20_000).saturating_sub(endowment - rent0))
				* 4;
			let contract = ContractInfoOf::<Test>::get(&addr).unwrap().get_alive().unwrap();
			assert_eq!(contract.rent_allowance, allowance - rent0 - rent);
			assert_eq!(contract.deduct_block, 5);
			assert_eq!(Balances::free_balance(&addr), endowment - rent0 - rent);

			assert_ok!(Contracts::call(
				Origin::signed(ALICE),
				addr.clone(),
				0,
				GAS_LIMIT,
				40000u32.to_le_bytes().to_vec(),
			));

			// Advance 1 blocks
			initialize_block(6);

			// Trigger rent through call
			assert_ok!(
				Contracts::call(Origin::signed(ALICE), addr.clone(), 0, GAS_LIMIT, call::null())
			);

			let contract = ContractInfoOf::<Test>::get(&addr).unwrap().get_alive().unwrap();
			let storageSize = contract.storage_size;
			let contractPairCount = contract.pair_count;

			let rentFraction = <Test as Config>::RentFraction::get();
			let gross_rent_price: u64 = (8 + (storageSize as u64) + code_len ) * 10_000 + ((contractPairCount as u64) * 10_000);
			let net_rent_price: u64 = gross_rent_price.saturating_sub(endowment -rent0 - rent);
			let rent2 = rentFraction.mul_ceil(net_rent_price)
				// blocks to rent
				* 1;
			assert_eq!(contract.rent_allowance, allowance - rent0 - rent - rent2);
			assert_eq!(Balances::free_balance(&addr), endowment - rent0 - rent - rent2);

			// Advance 1 blocks
			initialize_block(7);

			// A snitch can now remove the contract
			assert_ok!(Contracts::claim_surcharge(Origin::none(), addr.clone(), Some(ALICE)));
			assert!(ContractInfoOf::<Test>::get(&addr).unwrap().get_tombstone().is_some());
		});
}
```

2. Resore a contract:
```rust
#[test]
fn retrieve_storage_from_tombstone() {
	let (wasm, code_hash) = compile_module::<Test>("set_rent").unwrap();
	let (restoration_wasm, restoration_code_hash) = compile_module::<Test>("restoration").unwrap();

	// Balance reached and superior to subsistence threshold
	ExtBuilder::default()
		.existential_deposit(50)
		.build()
		.execute_with(|| {
			// Create
			let _ = Balances::deposit_creating(&ALICE, 1_000_000);
			assert_ok!(Contracts::instantiate_with_code(
				Origin::signed(ALICE),
				30_000,
				GAS_LIMIT,
				wasm.clone(),
				<Test as pallet_balances::Config>::Balance::from(10_000u32).encode(),
				vec![],
			));
			let addr = Contracts::contract_address(&ALICE, &code_hash, &[]);
			let allowance = ContractInfoOf::<Test>::get(&addr)
				.unwrap().get_alive().unwrap().rent_allowance;
			let balance = Balances::free_balance(&addr);

			assert_eq!(
				ContractInfoOf::<Test>::get(&addr).unwrap().get_alive().unwrap().rent_allowance,
				allowance,
			);
			assert_eq!(Balances::free_balance(&addr), balance);

			// Create another contract from the same code in order to increment the codes
			// refcounter so that it stays on chain.
			assert_ok!(Contracts::instantiate_with_code(
				Origin::signed(ALICE),
				20_000,
				GAS_LIMIT,
				wasm.clone(),
				<Test as pallet_balances::Config>::Balance::from(10_000u32).encode(),
				vec![1],
			));
			let addr_dummy = Contracts::contract_address(&ALICE, &code_hash, &[1]);

			// Advance blocks
			initialize_block(27);

			assert!(ContractInfoOf::<Test>::get(&addr).unwrap().get_alive().is_some());
			// A snitch can now remove the contract
			assert_ok!(Contracts::claim_surcharge(Origin::none(), addr.clone(), Some(ALICE)));
			assert!(ContractInfoOf::<Test>::get(&addr).unwrap().get_tombstone().is_some());


			let _ = Balances::deposit_creating(&CHARLIE, 1_000_000);
			assert_ok!(Contracts::instantiate_with_code(
				Origin::signed(CHARLIE),
				30_000,
				GAS_LIMIT,
				restoration_wasm.clone(),
				<Test as pallet_balances::Config>::Balance::from(10_000u32).encode(),
				vec![],
			));
			let addr_django = Contracts::contract_address(&CHARLIE, &restoration_code_hash, &[]);

			// Before performing a call to `DJANGO` save its original trie id.
			let django_trie_id = ContractInfoOf::<Test>::get(&addr_django).unwrap()
				.get_alive().unwrap().trie_id;

			// Advance 1 block.
			initialize_block(31);

			// Perform a call to `DJANGO` to perform restoration successfully
			assert_ok!(Contracts::call(
					Origin::signed(ALICE),
					addr_django.clone(),
					0,
					GAS_LIMIT,
					code_hash
						.as_ref()
						.iter()
						.chain(AsRef::<[u8]>::as_ref(&addr))
						.cloned()
						.collect(),
				));

			// Here we expect that the restoration is succeeded. Check that the restoration
			// contract `DJANGO` ceased to exist and that `BOB` returned back.
			let bob_contract = ContractInfoOf::<Test>::get(&addr).unwrap()
				.get_alive().unwrap();
			assert_eq!(bob_contract.rent_allowance, 50);
			assert_eq!(bob_contract.storage_size, 4);
			assert_eq!(bob_contract.trie_id, django_trie_id);
			assert_eq!(bob_contract.deduct_block, System::block_number());
			assert!(ContractInfoOf::<Test>::get(&addr_django).is_none());
		});
}
```


