#![no_std]

use soroban_sdk {
    contract, contracterror, contractimpl, contracttype, vec, Address, Env, String,
};

const STORAGE_FEE_PER_BYTE: u32 = 1000;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReferralError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    UnauthorizedCaller = 3,
    InvalidDisplayName = 10,
    IncompatibleInterface = 11,
    NameTooLong = 12,
    InsufficientFee = 13,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Profile(Address),
}

#[contract]
pub struct ReferralContract;

#[contractimpl]
impl ReferralContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), ReferralError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ReferralError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().extend_ttl(3_153_600, 6_307_200);
        Ok(())
    }

    pub fn register_referral(
        env: Env,
        caller: Address,
        display_name: String,
    ) -> Result<(), ReferralError> {
        caller.require_auth();

        if !env.storage().instance().has(&DataKey::Admin) {
            return Err(ReferralError::NotInitialized);
        }

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap();
        if caller != admin {
            return Err(ReferralError::UnauthorizedCaller);
        }

        let display_name_bytes: Vec<u8> = display_name.as_bytes().to_vec();
        let name_len = display_name_bytes.len();

        if name_len == 0 {
            return Err(ReferralError::InvalidDisplayName);
        }

        if name_len > 64 {
            return Err(ReferralError::NameTooLong);
        }

        let storage_fee = name_len as u64 * STORAGE_FEE_PER_BYTE;
        let caller_balance = env.bank().balance(&caller).0;

        if caller_balance < storage_fee {
            return Err(ReferralError::InsufficientFee);
        }

        let profile_key = DataKey::Profile(caller.clone());
        env.storage()
            .persistent()
            .set(&profile_key, &());
        env.storage()
            .persistent()
            .extend_ttl(&profile_key, 3_153_600, 6_307_200);

        let remaining_balance = caller_balance - storage_fee;
        let _ = env.bank().transfer(&caller, &remaining_balance);

        env.events().publish((
            Symbol::new(&env, "referral_registered"),
            caller,
            display_name,
        ));

        Ok(())
    }

    pub fn get_display_name(env: Env, user: Address) -> Option<String> {
        env.storage()
            .persistent()
            .get::<DataKey, String>(&DataKey::Profile(user))
    }

    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Admin)
    }
}