use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ADMIN: Item<Addr> = Item::new("admin");
pub const TOKEN: Item<Addr> = Item::new("token");
pub const TOKEN_COUNT: Item<u64> = Item::new("token_count");
