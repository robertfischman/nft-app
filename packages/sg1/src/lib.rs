use cosmwasm_std::{coins, Addr, BankMsg, Decimal, Event, MessageInfo, Uint128};
use cw_utils::{must_pay, PaymentError};
use sg_std::{create_fund_community_pool_msg, Response, SubMsg, NATIVE_DENOM};
use thiserror::Error;

// governance parameters
const FEE_BURN_PERCENT: u64 = 50;
const DEV_INCENTIVE_PERCENT: u64 = 10;

pub struct FairBurnEvent {
    pub burn_amount: Uint128,
    pub dev: Option<Addr>,
    pub dev_amount: Option<Uint128>,
    pub dist_amount: Uint128,
}

/// Burn and distribute fees and return an error if the fee is not enough
pub fn checked_fair_burn(
    info: &MessageInfo,
    fee: u128,
    developer: Option<Addr>,
    res: &mut Response,
) -> Result<(), FeeError> {
    let payment = must_pay(info, NATIVE_DENOM)?;
    if payment.u128() < fee {
        return Err(FeeError::InsufficientFee(fee, payment.u128()));
    };

    fair_burn(fee, developer, res);

    Ok(())
}

/// Burn and distribute fees, assuming the right fee is passed in
pub fn fair_burn(fee: u128, developer: Option<Addr>, res: &mut Response) {
    let (burn_percent, dev_fee) = match developer {
        Some(dev) => {
            let dev_fee = (Uint128::from(fee) * Decimal::percent(DEV_INCENTIVE_PERCENT)).u128();
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: dev.to_string(),
                amount: coins(dev_fee, NATIVE_DENOM),
            }));
            (
                Decimal::percent(FEE_BURN_PERCENT - DEV_INCENTIVE_PERCENT),
                dev_fee,
            )
        }
        None => (Decimal::percent(FEE_BURN_PERCENT), 0u128),
    };

    // burn half the fee
    let burn_fee = (Uint128::from(fee) * burn_percent).u128();
    let burn_coin = coins(burn_fee, NATIVE_DENOM);
    res.messages
        .push(SubMsg::new(BankMsg::Burn { amount: burn_coin }));

    // Send other half to community pool
    res.messages
        .push(SubMsg::new(create_fund_community_pool_msg(coins(
            fee - (burn_fee + dev_fee),
            NATIVE_DENOM,
        ))));
}

#[derive(Error, Debug, PartialEq)]
pub enum FeeError {
    #[error("Insufficient fee: expected {0}, got {1}")]
    InsufficientFee(u128, u128),

    #[error("{0}")]
    Payment(#[from] PaymentError),
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, BankMsg};

    use crate::{fair_burn, SubMsg};

    #[test]
    fn check_fair_burn_no_dev_rewards() {
        let msgs = fair_burn(1000u128, None);
        let burn_msg = SubMsg::Bank(BankMsg::Burn {
            amount: coins(500, "ustars".to_string()),
        });
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0], burn_msg);
    }

    #[test]
    fn check_fair_burn_with_dev_rewards() {
        let msgs = fair_burn(1000u128, Some(Addr::unchecked("geordi")));
        let bank_msg = SubMsg::Bank(BankMsg::Send {
            to_address: "geordi".to_string(),
            amount: coins(100, "ustars".to_string()),
        });
        let burn_msg = SubMsg::Bank(BankMsg::Burn {
            amount: coins(400, "ustars".to_string()),
        });
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0], bank_msg);
        assert_eq!(msgs[1], burn_msg);
    }

    #[test]
    fn check_fair_burn_with_dev_rewards_different_amount() {
        let mut res = Response::new();

        fair_burn(1420u128, Some(Addr::unchecked("geordi")), &mut res);
        let bank_msg = SubMsg::new(BankMsg::Send {
            to_address: "geordi".to_string(),
            amount: coins(710, NATIVE_DENOM),
        });
        let burn_msg = SubMsg::new(BankMsg::Burn {
            amount: coins(710, NATIVE_DENOM),
        });
        assert_eq!(res.messages.len(), 2);
        assert_eq!(res.messages[0], bank_msg);
        assert_eq!(res.messages[1], burn_msg);
    }
}
