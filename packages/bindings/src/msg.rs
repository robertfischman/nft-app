use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::route::StargazeRoute;
use cosmwasm_std::{Coin, CosmosMsg, CustomMsg};

static MSG_DATA_VERSION: &str = "1.0.0";

/// StargazeMsg is an override of CosmosMsg::Custom to add support for Stargaze's custom message types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StargazeMsgWrapper {
    pub route: StargazeRoute,
    pub msg_data: StargazeMsg,
    pub version: String,
}

impl From<StargazeMsgWrapper> for CosmosMsg<StargazeMsgWrapper> {
    fn from(original: StargazeMsgWrapper) -> Self {
        CosmosMsg::Custom(original)
    }
}

impl CustomMsg for StargazeMsgWrapper {}

/// StargazeMsg is an override of CosmosMsg::Custom to add support for Stargaze's custom message types
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StargazeMsg {
    ClaimFor {
        address: String,
        action: ClaimAction,
    },
    FundCommunityPool {
        amount: Vec<Coin>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClaimAction {
    MintNFT,
    BidNFT,
}

pub fn create_claim_for_msg(address: String, action: ClaimAction) -> CosmosMsg<StargazeMsgWrapper> {
    StargazeMsgWrapper {
        route: StargazeRoute::Claim,
        msg_data: StargazeMsg::ClaimFor { address, action },
        version: MSG_DATA_VERSION.to_owned(),
    }
    .into()
}

pub fn create_fund_community_pool_msg(amount: Vec<Coin>) -> CosmosMsg<StargazeMsgWrapper> {
    StargazeMsgWrapper {
        route: StargazeRoute::Distribution,
        msg_data: StargazeMsg::FundCommunityPool { amount },
        version: MSG_DATA_VERSION.to_owned(),
    }
    .into()
}
