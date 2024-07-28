#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod tarot {
    use ink::env::hash::{
        HashOutput,
        Blake2x128,
    };
    use ink::storage::Mapping;
    use ink::prelude::vec::Vec;

    // TarotDraw is an event for tracking a Tarot Reading.
    // The drawing field will have at least 1 but no more than 3
    // cards drawn in any given Drawing.
    #[ink::event]
    pub struct TarotDraw {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        drawing: Vec<[u8; 16]>
    }


    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Tarot {
        /// owner is the account id owner that can collect payments to
        /// this contract.
        owner: AccountId,
        /// fee is the minimum fee that mus be paid to draw a card
        fee: Balance,
        /// readings stores a mapping of AccountId's to drawings
        readings: Mapping<AccountId, Vec<[u8; 16]>>
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[derive(Debug, PartialEq, Eq)]
    pub enum Error {
        InsufficientFee,
        MustBeOwner,
    }

    impl Tarot {
        /// new initializes an empty mapping of readings.
        #[ink(constructor)]
        pub fn new(fee: Balance) -> Self {
            Tarot {
                readings: Mapping::default(),
                fee,
                owner: Self::env().caller()
            }
        }

        #[ink(message, payable)]
        pub fn draw(&mut self, seed: [u8; 16]) -> Result<[u8; 16], Error> {
            let caller = self.env().caller();

            if self.env().transferred_value() < self.fee {
                return Err(Error::InsufficientFee);
            }

            let ts = self.env().block_timestamp();
            let bn = self.env().block_number();
            let cb = self.env().balance();
            let gl = self.env().gas_left();
            let ch = self.env().own_code_hash().unwrap();

            let encodable = (ts, bn, cb, gl, ch, seed);

            let mut draw = <Blake2x128 as HashOutput>::Type::default();
            ink::env::hash_encoded::<Blake2x128, _>(&encodable,
                                                    &mut draw);

            let mut current: Vec<[u8; 16]> = self.readings.get(caller).unwrap_or(
                Vec::new());
            if current.len() >= 3 {
                current = Vec::new();
            }
            current.push(draw);

            self.readings.insert(caller, &current);

            Ok(draw)
        }

        /// change_owner change the contract owner
        #[ink(message)]
        pub fn change_owner(&mut self, new_owner: AccountId) -> Result<(), Error> {
            let caller = self.env().caller();
            if caller != self.owner {
                return Err(Error::InsufficientFee);
            }
            self.owner = new_owner;
            Ok(())
        }

        /// fee simply returns the current fee
        #[ink(message)]
        pub fn fee(&self) -> Balance {
            self.fee
        }

        /// withdraw balance to the contract owner.
        #[ink(message)]
        pub fn withdraw(&mut self) {
            let balance = self.env().balance();
            self.env().transfer(self.owner, balance).unwrap()
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can
        /// use them here.
        use super::*;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut t = Tarot::new(100);
            assert_eq!(t.fee(), 100);
            // assert_eq!(t.fee(), 100);
        }
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = TarotRef::new(100);
            let contract = client
                .instantiate("tarot", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Tarot>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), 100));

            // When
            let flip = call_builder.fee();
            let _flip_result = client
                .call(&ink_e2e::bob(), &fee)
                .submit()
                .await
                .expect("fee failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), 100));

            Ok(())
        }
    }
}
