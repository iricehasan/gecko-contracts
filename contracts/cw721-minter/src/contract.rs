#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, wasm_instantiate, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply,
    Response, StdResult, SubMsg, WasmMsg,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, Metadata, QueryMsg};
use crate::state::{ADMIN, TOKEN, TOKEN_COUNT};

use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use cw721_base::InstantiateMsg as Cw721InstantaiteMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const INSTANTIATE_REPLY: u64 = 1;
pub const MINT_REPLY: u64 = 2;

pub type Extension = Option<Metadata>;
pub type Cw721BaseExecuteMsg = cw721_base::ExecuteMsg<Extension, Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cw721_init_msg = Cw721InstantaiteMsg {
        name: msg.name,
        symbol: msg.symbol,
        minter: env.contract.address.to_string(),
    };

    let submsg = SubMsg::reply_on_success(
        wasm_instantiate(
            msg.cw721_code_id,
            &cw721_init_msg,
            vec![],
            "Nft Contract".to_owned(),
        )
        .unwrap(),
        INSTANTIATE_REPLY,
    );

    TOKEN.save(deps.storage, &Addr::unchecked(""))?;
    TOKEN_COUNT.save(deps.storage, &0u64)?; 

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_submessage(submsg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MintToken {
            recipient,
            token_uri,
            extension,
        } => {

            let token_addr = TOKEN.load(deps.storage)?;
            let mut token_count = TOKEN_COUNT.load(deps.storage)?;

            let token_id = token_count.to_string();
            let submsg_mint = SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: token_addr.clone().to_string(),
                    msg: to_json_binary(&Cw721BaseExecuteMsg::Mint {
                        token_id: token_id,
                        owner: recipient.to_string(),
                        token_uri: token_uri,
                        extension: extension,
                    })?,
                    funds: vec![],
                },
                MINT_REPLY,
            );

            token_count += 1;
            TOKEN_COUNT.save(deps.storage, &token_count)?;

            Ok(Response::new()
                .add_attribute("action", "mint")
                .add_submessage(submsg_mint))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        INSTANTIATE_REPLY => {
            let res = parse_reply_instantiate_data(reply).unwrap();
            let contract_address = deps.api.addr_validate(&res.contract_address).unwrap();
            TOKEN.save(deps.storage, &contract_address)?;

            Ok(Response::default())
        }
        MINT_REPLY => Ok(Response::new().add_attribute("Operation", "mint")),
        _ => Err(ContractError::UnrecognizedReply {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Token {} => Ok(to_json_binary(&TOKEN.load(deps.storage)?)?),
    }
}
