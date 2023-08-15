
#[derive(Debug, PartialEq)] 
struct LpPool {
    token_reserve: u64,
    staked_token_reserve: u64,
    lp_token_supply: u64,
    price: u64,
    fee_min: u64,
    fee_max: u64,
    liquidity_target: u64,
}

#[derive(Debug, PartialEq)]
enum Errors {
    PropertyMustBeGreaterThanZero,
    FeeMaxMustBeGreaterThanFeeMin,
    InsufficientLiquidity
}

impl std::fmt::Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Errors::PropertyMustBeGreaterThanZero => write!(f, "Property must be greater than zero"),
            Errors::FeeMaxMustBeGreaterThanFeeMin => write!(f, "Fee max must be greater than fee min"),
            Errors::InsufficientLiquidity => write!(f, "Insufficient liquidity"),
        }
    }
}

impl LpPool {

    fn init(price: u64, fee_min: u64, fee_max: u64, liquidity_target: u64) -> Result<Self, Errors> {
        if price == 0 || fee_min == 0 || fee_max == 0 || liquidity_target == 0 {
            return Err(Errors::PropertyMustBeGreaterThanZero);
        }
        
        if fee_min >= fee_max {
            return Err(Errors::FeeMaxMustBeGreaterThanFeeMin);
        }
        
        Ok(LpPool {
            token_reserve: 0,
            staked_token_reserve: 0,
            lp_token_supply: 0,
            price,
            fee_min,
            fee_max,
            liquidity_target,
        })

    }

    fn add_liquidity(&mut self, amount: u64) -> Result<u64, Errors> {
        if amount == 0 {
            return Err(Errors::PropertyMustBeGreaterThanZero);
        }

        self.token_reserve += amount;

        let liquidity_minted = if self.lp_token_supply == 0 {
            amount
        } else {
            (amount as f64 * (self.lp_token_supply as f64  / self.token_reserve as f64 ))  as u64
        };
        
        self.lp_token_supply += liquidity_minted;
        Ok(liquidity_minted)
    }

    fn remove_liquidity(&mut self, lp_token_amount: u64) ->  Result<(u64, u64), Errors> {
        if lp_token_amount == 0 {
            return Err(Errors::PropertyMustBeGreaterThanZero);
        }

        if lp_token_amount > self.lp_token_supply {
            return Err(Errors::InsufficientLiquidity);
        }

        let token_amount = ((lp_token_amount * self.token_reserve) as f64 / self.lp_token_supply as f64) as u64;
        let staked_token_amount = ((lp_token_amount * self.staked_token_reserve) as f64 / self.lp_token_supply as f64)  as u64;
        
        if token_amount > self.token_reserve || staked_token_amount > self.staked_token_reserve   {
            return Err(Errors::InsufficientLiquidity);
        }

        self.token_reserve -= token_amount;
        self.staked_token_reserve -= staked_token_amount;
        self.lp_token_supply -= lp_token_amount;

        Ok((token_amount, staked_token_amount))
    }

    fn swap(&mut self, staked_token_amount: u64) -> Result<u64, Errors> {
        if staked_token_amount ==0  {
            return Err(Errors::PropertyMustBeGreaterThanZero);
        }

        let token_amount = staked_token_amount * self.price;
        let fee_percentage = self.calculate_fee_percentage();
        let fee = (token_amount * fee_percentage) / 100;

        if token_amount > self.token_reserve {
            return Err(Errors::InsufficientLiquidity);
        }

        self.token_reserve -= token_amount;
        self.staked_token_reserve += staked_token_amount;

        Ok(token_amount - fee)
    }

    fn calculate_fee_percentage(&self) -> u64 {
        let liquidity_ratio = (self.token_reserve * 100) / self.liquidity_target;
        self.fee_min + ((liquidity_ratio * (self.fee_max - self.fee_min)) / 100)
    }
}

fn main() {
    println!("---");

    let mut lp_pool = LpPool::init(5, 1, 9, 1000).unwrap();
    let add_liquidity_result1  = lp_pool.add_liquidity(10).unwrap();
    println!("Minted 1 :: {}",add_liquidity_result1);
    
    let add_liquidity_result2  = lp_pool.add_liquidity(20).unwrap();
    println!("Minted 2 :: {}",add_liquidity_result2);
    
    let swap1 = lp_pool.swap(3).unwrap();
    println!("Tokens received from swap 1: {}", swap1);

    let (tokens_returned, staked_tokens_returned) = lp_pool.remove_liquidity(10).unwrap();
    println!("Tokens returned: {}, Staked Tokens returned: {}", tokens_returned, staked_tokens_returned);
}


#[cfg(test)]
mod tests {
    use super::*;    
    
    // init

    #[test]
    fn test_init_success() {
        let lp_pool = LpPool::init(100, 5, 6, 1000);
        assert!(lp_pool.is_ok());
    }

    #[test]
    fn test_init_zero_price() {
        let lp_pool = LpPool::init(0, 5, 1, 1000);
        assert_eq!(lp_pool, Err(Errors::PropertyMustBeGreaterThanZero));
    }

    #[test]
    fn test_init_zero_fee_min() {
        let lp_pool = LpPool::init(100, 0, 1, 1000);
        assert_eq!(lp_pool, Err(Errors::PropertyMustBeGreaterThanZero));
    }

    #[test]
    fn test_init_zero_fee_max() {
        let lp_pool = LpPool::init(100, 5, 0, 1000);
        assert_eq!(lp_pool, Err(Errors::PropertyMustBeGreaterThanZero));
    }

    #[test]
    fn test_init_zero_liquidity_target() {
        let lp_pool = LpPool::init(100, 5, 1, 0);
        assert_eq!(lp_pool, Err(Errors::PropertyMustBeGreaterThanZero));
    }

    #[test]
    fn test_init_fee_min_greater_than_fee_max() {
        let lp_pool = LpPool::init(100, 5, 4, 1000);
        assert_eq!(lp_pool, Err(Errors::FeeMaxMustBeGreaterThanFeeMin));
    }

    #[test]
    fn test_init_fee_min_equal_to_fee_max() {
        let lp_pool = LpPool::init(100, 5, 5, 1000);
        assert_eq!(lp_pool, Err(Errors::FeeMaxMustBeGreaterThanFeeMin));
    }

    #[test]
    fn test_init_all_properties_zero() {
        let lp_pool = LpPool::init(0, 0, 0, 0);
        assert_eq!(lp_pool, Err(Errors::PropertyMustBeGreaterThanZero));
    }

    // add_liquidity

    #[test]
    fn test_add_liquidity_first_time() {
        let mut lp_pool = LpPool::init(100, 5, 10, 1000).unwrap();
        let liquidity_added: Result<u64, Errors> = lp_pool.add_liquidity(200);
        assert_eq!(liquidity_added, Ok(200));
        assert_eq!(lp_pool.lp_token_supply, 200);
        assert_eq!(lp_pool.token_reserve, 200);
    }

    #[test]
    fn test_add_liquidity_token_reserve() {
        let mut lp_pool = LpPool::init(100, 5, 10, 1000).unwrap();
        let _ =  lp_pool.add_liquidity(200);        
        let _ =  lp_pool.add_liquidity(300);
        assert_eq!(lp_pool.token_reserve, 500);
        
    }
    #[test]
    fn test_add_liquidity_minted_tokens_twice() {
        let mut lp_pool = LpPool::init(100, 5, 10, 1000).unwrap();
        let minted_tokens1 =  lp_pool.add_liquidity(200);        
        assert_eq!(minted_tokens1, Ok(200));
        let minted_tokens2 =  lp_pool.add_liquidity(300);
        assert_eq!(minted_tokens2, Ok(120));
        
    }

    #[test]
    fn test_add_liquidity_zero_amount() {
        let mut lp_pool = LpPool::init(100, 5, 10, 1000).unwrap();
        let result = lp_pool.add_liquidity(0);
        assert_eq!(result, Err(Errors::PropertyMustBeGreaterThanZero));
        assert_eq!(lp_pool.lp_token_supply, 0);
        assert_eq!(lp_pool.token_reserve, 0);
    }

    // remove_liquidity

    #[test]
    fn test_remove_liquidity_successful() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        lp_pool.token_reserve = 200;
        lp_pool.staked_token_reserve = 300;
        lp_pool.lp_token_supply = 500;
        let result = lp_pool.remove_liquidity(100).unwrap();
        assert_eq!(result, (40, 60));
        assert_eq!(lp_pool.token_reserve, 160);
        assert_eq!(lp_pool.staked_token_reserve, 240);
        assert_eq!(lp_pool.lp_token_supply, 400);
    }

    #[test]
    fn test_remove_zero_lp_tokens() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        let result = lp_pool.remove_liquidity(0);
        assert_eq!(result, Err(Errors::PropertyMustBeGreaterThanZero));
    }

    #[test]
    fn test_remove_insufficient_lp_tokens() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        lp_pool.lp_token_supply = 500;
        let result = lp_pool.remove_liquidity(600);
        assert_eq!(result, Err(Errors::InsufficientLiquidity));
    }

    #[test]
    fn test_remove_with_zero_reserves() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        let result = lp_pool.remove_liquidity(100);
        assert_eq!(result, Err(Errors::InsufficientLiquidity));
    }

    #[test]
    fn test_remove_partial_tokens() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        lp_pool.token_reserve = 200;
        lp_pool.staked_token_reserve = 300;
        lp_pool.lp_token_supply = 500;
        let result = lp_pool.remove_liquidity(50).unwrap();
        assert_eq!(result, (20, 30));
        assert_eq!(lp_pool.token_reserve, 180);
        assert_eq!(lp_pool.staked_token_reserve, 270);
        assert_eq!(lp_pool.lp_token_supply, 450);
    }

    #[test]
    fn test_remove_full_supply() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        lp_pool.token_reserve = 200;
        lp_pool.staked_token_reserve = 300;
        lp_pool.lp_token_supply = 500;
        let result = lp_pool.remove_liquidity(500).unwrap();
        assert_eq!(result, (200, 300));
        assert_eq!(lp_pool.token_reserve, 0);
        assert_eq!(lp_pool.staked_token_reserve, 0);
        assert_eq!(lp_pool.lp_token_supply, 0);
    }

    // swap

    #[test]
    fn test_swap_with_sufficient_liquidity() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        lp_pool.token_reserve = 1000;
        lp_pool.staked_token_reserve = 100;
        
        let result = lp_pool.swap(10).unwrap();
        assert_eq!(result, 980);
    }

    #[test]
    fn test_swap_with_insufficient_liquidity() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();

        let _ = lp_pool.add_liquidity(1000);
        let result = lp_pool.swap(11);
        assert_eq!(result, Err(Errors::InsufficientLiquidity));
    }

    #[test]
    fn test_swap_with_zero_provided() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        
        let result = lp_pool.swap(0);
        assert_eq!(result, Err(Errors::PropertyMustBeGreaterThanZero));
    }

    #[test]
    fn test_swap_with_zero_token_reserve() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        let result = lp_pool.swap(10);
        assert_eq!(result, Err(Errors::InsufficientLiquidity));
    }

    #[test]
    fn test_swap_with_fee_and_liquidity_ratio() {
        let mut lp_pool = LpPool::init(100, 1, 2, 1000).unwrap();
        lp_pool.token_reserve = 1000;
        lp_pool.staked_token_reserve = 100;
        lp_pool.lp_token_supply = 1000;
        
        let result = lp_pool.swap(10).unwrap();
        assert_eq!(result, 980);
    }

    // fee_calculation        

    #[test]
    fn test_fee_calculation_at_min() {
        let lp_pool = LpPool {
            token_reserve: 1000,
            staked_token_reserve: 500,
            lp_token_supply: 100,
            price: 10,
            fee_min: 1,
            fee_max: 5,
            liquidity_target: 2000,
        };

        let fee_percentage = lp_pool.calculate_fee_percentage();
        assert_eq!(fee_percentage, 3);
    }

    #[test]
    fn test_fee_calculation_at_max() {
        let lp_pool = LpPool {
            token_reserve: 1000,
            staked_token_reserve: 500,
            lp_token_supply: 100,
            price: 10,
            fee_min: 1,
            fee_max: 5,
            liquidity_target: 500,
        };

        let fee_percentage = lp_pool.calculate_fee_percentage();
        assert_eq!(fee_percentage, 9);
    }

    #[test]
    fn test_fee_calculation_below_min() {
        let lp_pool = LpPool {
            token_reserve: 1000,
            staked_token_reserve: 500,
            lp_token_supply: 100,
            price: 10,
            fee_min: 1,
            fee_max: 5,
            liquidity_target: 5000,
        };

        let fee_percentage = lp_pool.calculate_fee_percentage();
        assert_eq!(fee_percentage, 1);
    }

    #[test]
    fn test_fee_calculation_with_zero_reserves() {
        let lp_pool = LpPool {
            token_reserve: 100,
            staked_token_reserve: 0,
            lp_token_supply: 0,
            price: 10,
            fee_min: 1,
            fee_max: 5,
            liquidity_target: 1000,
        };

        let fee_percentage = lp_pool.calculate_fee_percentage();
        assert_eq!(fee_percentage, 1);
    }

}