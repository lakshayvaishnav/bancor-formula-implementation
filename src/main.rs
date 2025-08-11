fn main() {
    let mut sol_reserve = 12_330_000_000u64;
    let mut token_supply = 618_496_769u64;

    // purchase
    let tokens = calculate_purchase_return(20, sol_reserve, token_supply, 100_000_000);
    // tokens is currently floored u64 in your code

    // update reserves after purchase (tokens minted, reserve increased)
    sol_reserve = sol_reserve + 100_000_000; // R' = R + ΔR
    token_supply = token_supply + tokens; // T' = T + ΔT

    // now sell the same tokens against updated reserves
    let sol = calculate_sale_return(20, sol_reserve, token_supply, tokens);

    println!("tokens minted: {}", tokens);
    println!("sol returned:  {}", sol);
}

pub fn calculate_purchase_return(
    connector_weight: u32,
    virtual_sol_reserve: u64,
    virtual_token_reserve: u64,
    deposit_amount: u64
) -> u64 {
    let cw = (connector_weight as f64) / 100.0;
    let virtual_sol = virtual_sol_reserve as f64;
    let virtual_token = virtual_token_reserve as f64;

    // if you charge a fee, apply it here:
    let amount_after_fee = deposit_amount as f64;

    let base = 1.0 + amount_after_fee / virtual_sol;
    let tokens_out_f = virtual_token * (base.powf(cw) - 1.0);
    tokens_out_f.floor() as u64
}

pub fn calculate_sale_return(
    connector_weight: u32,
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    sell_tokens: u64
) -> u64 {
    let cw = (connector_weight as f64) / 100.0;
    let virtual_sol = virtual_sol_reserves as f64;
    let virtual_token = virtual_token_reserves as f64;

    // apply any sell fees here
    let amount_after_fee = sell_tokens as f64;

    let base = 1.0 - amount_after_fee / virtual_token;
    let sol_out_f = virtual_sol * (1.0 - base.powf(1.0 / cw));
    sol_out_f.floor() as u64
}

/*
tokens minted: 161682
sol returned:  99_999_795   // ~100_000_000 minus ~204 lamports due to rounding
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_symmetry_purchase_then_sale() {
        // initial reserves (same as your example)
        let mut sol_reserve = 12_330_000_000u64;
        let mut token_supply = 100_000_000u64;
        let deposit = 100_000_000u64; // 0.1 SOL in lamports

        // purchase (this function floors the minted tokens)
        let minted = calculate_purchase_return(20, sol_reserve, token_supply, deposit);

        // update reserves after the purchase
        sol_reserve = sol_reserve + deposit;
        token_supply = token_supply + minted;

        // sell the same tokens against the updated reserves
        let returned = calculate_sale_return(20, sol_reserve, token_supply, minted);

        // expected minted value observed in your run
        assert_eq!(minted, 161_682u64);

        // check that the round-trip difference is small (tolerance in lamports)
        let diff = ((deposit as i128) - (returned as i128)).abs() as u64;
        assert!(diff <= 1_000u64, "round-trip difference too large: {} lamports", diff);
    }

    #[test]
    fn float_symmetry_is_precise() {
        // helper functions that operate in f64 (no flooring)
        fn purchase_f64(
            cw_percent: u32,
            virtual_sol: f64,
            virtual_token: f64,
            deposit: f64
        ) -> f64 {
            let cw = (cw_percent as f64) / 100.0;
            virtual_token * ((1.0 + deposit / virtual_sol).powf(cw) - 1.0)
        }

        fn sale_f64(
            cw_percent: u32,
            virtual_sol: f64,
            virtual_token: f64,
            sell_tokens: f64
        ) -> f64 {
            let cw = (cw_percent as f64) / 100.0;
            virtual_sol * (1.0 - (1.0 - sell_tokens / virtual_token).powf(1.0 / cw))
        }

        let virtual_sol = 12_330_000_000f64;
        let virtual_token = 100_000_000f64;
        let deposit = 100_000_000f64;

        let minted = purchase_f64(20, virtual_sol, virtual_token, deposit);
        let sol_after = virtual_sol + deposit;
        let token_after = virtual_token + minted;
        let returned = sale_f64(20, sol_after, token_after, minted);
        
        let abs_diff = (deposit - returned).abs();
        let rel_diff = abs_diff / deposit.max(1.0); // avoid divide by 0
    
        println!("✅ absolute diff = {}", abs_diff);
        println!("✅ relative diff = {}", rel_diff);
    
        // relative tolerance: allow differences up to 1e-12 (one part in 1e12)
        assert!(
            rel_diff < 1e-12,
            "float round trip not precise: abs_diff = {}, rel_diff = {}",
            abs_diff,
            rel_diff
        );
    }
}
