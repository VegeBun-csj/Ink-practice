#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::{
        collections::HashMap,
        lazy::Lazy,
    };

    #[ink(storage)]
    pub struct Erc20 {
        // 单值用lazy
        total_supply: Lazy<Balance>,
        balances: HashMap<AccountId, Balance>,
        allowances: HashMap<(AccountId, AccountId), Balance>
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: Balance
    }

    #[ink(event)]
    pub struct TransferFrom {
        #[ink(topic)]
        spender: Option<AccountId>,
        #[ink(topic)]
        from: Option<AccountId>,
        to: Option<AccountId>,
        value: Balance
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance
    }

    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientApproval
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(init_value: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = HashMap::new();
            balances.insert(caller, init_value);

            Self::env().emit_event(
                Transfer {
                    from: None,
                    to: Some(caller),
                    value: init_value
                }
            );

            Self {
                total_supply: Lazy::new(init_value),
                balances,
                allowances: HashMap::new()
            }
        }


        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }


        #[ink(message)]
        pub fn total_supply(&self) -> Balance{
            *self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, who: AccountId) -> Balance {
            self.balances.get(&who).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()>{
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);

            Self::env().emit_event(
                Approval {
                    owner,
                    spender,
                    value
                }
            );

            Ok(())
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let from = self.env().caller();
            self.inter_transfer(from, to, value)
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);

            if value > allowance {
                return Err(Error::InsufficientApproval);
            }

            self.inter_transfer(from, to, value)?;

            Self::env().emit_event(
                TransferFrom {
                    spender: Some(caller),
                    from: Some(from),
                    to: Some(to),
                    value
                }
            );

            let new_allowance = allowance - value;

            self.allowances.insert((from, caller), new_allowance);

            Self::env().emit_event(
                Approval {
                    owner: from,
                    spender: caller,
                    value
                }
            );

            Ok(())
        }

        fn inter_transfer(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let from_balance: Balance = self.balances.get(&from).copied().unwrap_or(0);

            if value > from_balance {
                return Err(Error::InsufficientBalance);
            }

            let to_balance = self.balances.get(&to).copied().unwrap_or(0);

            self.balances.insert(from, from_balance - value);
            self.balances.insert(to, to_balance + value);

            Self::env().emit_event(
                Transfer {
                    from: Some(from),
                    to: Some(to),
                    value
                }
            );

            Ok(())
        }

    }
}
