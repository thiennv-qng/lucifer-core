use crate::constant::*;
use crate::f64_trait::F64Trait;
use num_traits::ToPrimitive;

pub const WEIGHTS: [&'static u64; 2] = [&50, &50];

pub fn calc_normalize_weight(weight_idx: usize) -> Option<f64> {
  let total_weight: u64 = 100;
  let weight = WEIGHTS[weight_idx].to_f64()?;
  return Some(weight.checked_div(total_weight.to_f64()?)?);
}
fn complement(value: f64) -> Option<f64> {
  if value < 1_f64 {
    return Some(1_f64 - value);
  }
  return Some(0_f64);
}

pub fn calc_starting_lpt(amount_mint: u64, amount_stable_mint: u64) -> Option<u64> {
  let amounts_in: Vec<u64> = vec![amount_mint, amount_stable_mint];
  let mut lpt_amount = WEIGHTS.len().to_f64()?;
  for idx in 0..amounts_in.len() {
    let weight = calc_normalize_weight(idx)?;
    let amount_int = amounts_in[idx].to_f64()?;
    let new_val = amount_int.checked_pow(weight)?;
    lpt_amount = lpt_amount.checked_mul(new_val)?;
  }
  return Some(lpt_amount.to_u64()?);
}

// Return LPT amount will receive after add liquidity full side
pub fn calc_lpt_receive_add_full_side(
  supply: u64,
  amounts_in: &Vec<u64>,
  reserves: &Vec<u64>,
  total_fee: u64,
) -> Option<u64> {
  let mut balance_ratios_with_fee: Vec<f64> = Vec::new();
  let mut invariant_ratio_with_fees: f64 = 0_f64;

  for idx in 0..amounts_in.len() {
    let normalize_weight = calc_normalize_weight(idx)?;
    let balance = reserves[idx].to_f64()?;
    let amount_in = amounts_in[idx].to_f64()?;
    // (balance + amount) / balance
    let balance_ratio: f64 = (balance.checked_add(amount_in)?).checked_div(balance)?;
    balance_ratios_with_fee.push(balance_ratio);
    invariant_ratio_with_fees =
      invariant_ratio_with_fees.checked_add(balance_ratio.checked_mul(normalize_weight)?)?;
  }

  let mut invariant_ratio = 1_f64;

  for idx in 0..amounts_in.len() {
    let balance = reserves[idx].to_f64()?;
    let amount_in = amounts_in[idx].to_f64()?;
    let normalize_weight = calc_normalize_weight(idx)?;

    let mut amount_in_without_fee = amount_in;

    if balance_ratios_with_fee[idx] > invariant_ratio_with_fees {
      let non_taxable_amount =
        balance.checked_mul(invariant_ratio_with_fees.checked_sub(1_f64)?)?;
      let taxable_amount = amount_in.checked_sub(non_taxable_amount)?;
      let fee_ratio = total_fee.to_f64()?.checked_div(PRECISION)?;
      amount_in_without_fee = non_taxable_amount
        .checked_add(taxable_amount.checked_mul(1_f64.checked_sub(fee_ratio)?)?)?;
    }

    let balance_ratio = ((balance.checked_add(amount_in_without_fee))?).checked_div(balance)?;
    invariant_ratio = invariant_ratio.checked_mul(balance_ratio.checked_pow(normalize_weight)?)?;
  }

  if invariant_ratio > 1_f64 {
    let lpt_out = supply
      .to_f64()?
      .checked_mul(invariant_ratio.checked_sub(1_f64)?)?;
    return Some(lpt_out.to_u64()?);
  }
  return Some(0);
}

// Return mint amount will receive after remove liquidity single side
/*****************************************************************************************
// exactBPTInForTokenOut                                                                //
// a = amountOut                                                                        //
// b = balance                     /      /    supplyLPT - bptIn       \   (1 / w)  \   //
// bptIn = bptAmountIn    a = b * |  1 - | --------------------------  | ^           |  //
// bpt = supplyLPT                 \      \       supplyLPT            /            /   //
// w = weight                                                                           //
// _tbt = (  supply - bptIn ) / supplyLPT = 1 - (bptIn / supplyLPT)                     //
 *****************************************************************************************/
pub fn calc_mint_receive_remove_single_side(
  lpt_amount: u64,
  supply_lpt: u64,
  normalize_weight: f64,
  balance: u64,
  fee: u64,
) -> Option<u64> {
  let _tbt =
    (supply_lpt.to_f64()?.checked_sub(lpt_amount.to_f64()?)?).checked_div(supply_lpt.to_f64()?)?;
  let amount_out_without_fee = balance
    .to_f64()?
    .checked_mul(1_f64.checked_sub(_tbt.checked_pow(1_f64.checked_div(normalize_weight)?)?)?)?;

  let taxable_percentage = complement(normalize_weight)?;
  let taxable_amount = amount_out_without_fee.checked_mul(taxable_percentage)?;
  let non_taxable_amount = amount_out_without_fee.checked_sub(taxable_amount)?;
  let fee_rate = fee.to_f64()?.checked_div(PRECISION)?;

  let amount_out =
    non_taxable_amount.checked_add(taxable_amount.checked_mul(complement(fee_rate)?)?)?;
  return Some(amount_out.to_u64()?);
}

// Return list mint amount will receive after remove liquidity full side
pub fn calc_mint_receives_remove_full_side(
  lpt_amount: u64,
  supply: u64,
  reserves: &Vec<u64>,
) -> Option<Vec<u64>> {
  if lpt_amount > supply {
    return None;
  }
  let lpt_rate = lpt_amount.to_f64()?.checked_div(supply.to_f64()?)?;
  let mut amounts_out: Vec<u64> = Vec::new();

  for idx in 0..reserves.len() {
    let amount_out = lpt_rate.checked_mul(reserves[idx].to_f64()?)?;
    amounts_out.push(amount_out.to_u64()?)
  }
  return Some(amounts_out);
}

pub fn calc_ask_amount_swap(
  bid_amount: u64,
  bid_reserve: u64,
  ask_reserve: u64,
  fee: u64,
) -> Option<u64> {
  let bid_weight = calc_normalize_weight(1)?;
  let ask_weight = bid_weight;
  let _bi_bi_ai = bid_reserve
    .to_f64()?
    .checked_div(bid_reserve.to_f64()?.checked_add(bid_amount.to_f64()?)?)?;
  let _wi_wo = bid_weight.checked_div(ask_weight)?;
  let ask_amount = ask_reserve
    .to_f64()?
    .checked_mul(1_f64.checked_sub(_bi_bi_ai.checked_pow(_wi_wo)?)?)?;
  // fee
  let total_fee = fee.to_f64()?.checked_div(PRECISION)?;
  return Some((ask_amount.checked_mul(1_f64.checked_sub(total_fee)?)?).to_u64()?);
}
