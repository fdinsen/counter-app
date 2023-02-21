
pub fn probability_within_n_tries(n: i32, p: f64) -> f64 {
    1.0-(1.0-p).powf(n.into())
}

pub fn num_tries_for_x_percent_chance(x: f64, p: f64) -> i32 { 
    log_with_base(1.0-p, 1.0-x).round() as i32
}

pub fn convert_to_percentage(x:f64) -> f64 {
    x * 100.0
}

fn log_with_base(base: f64, x: f64) -> f64 {
    f64::ln(x) / f64::ln(base)
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;

    use super::{log_with_base, probability_within_n_tries, num_tries_for_x_percent_chance};

    #[test]
    fn test_log_with_base() {
        let base = 10.0;
        let x = 5.0;
        let exp = f64::ln(x);
        let res = log_with_base(base, x);
        println!("base: {}, x: {}, exp: {}, res: {}", base, x, exp, res);
    }

    #[test]
    fn test_log_with_base_5_of_6() {
        let base = 5.0;
        let x = 6.0;
        let exp = 1.11328; // https://www.symbolab.com/solver/equation-calculator/%5Clog_%7B5%7D%5Cleft(6%5Cright)?or=input
        let res = log_with_base(base, x);
        println!("base: {}, x: {}, exp: {}, res: {}", base, x, exp, res);
        assert!(approx_eq!(f64, exp, res, epsilon=0.00001))
    }

    #[test]
    fn test_log_with_base_6_251_of_3() {
        let base = 6.251;
        let x = 3.0;
        let exp = 0.59943; // https://www.symbolab.com/solver/equation-calculator/%5Clog_%7B6.251%7D%5Cleft(3%5Cright)?or=input
        let res = log_with_base(base, x);
        println!("base: {}, x: {}, exp: {}, res: {}", base, x, exp, res);
        assert!(approx_eq!(f64, exp, res, epsilon=0.00001))
    }

    #[test]
    fn test_log_with_base_0_562_of_26() {
        let base = 0.562;
        let x = 26.0;
        let exp = -5.65392; // https://www.symbolab.com/solver/equation-calculator/%5Clog_%7B0.562%7D%5Cleft(26%5Cright)?or=input
        let res = log_with_base(base, x);
        println!("base: {}, x: {}, exp: {}, res: {}", base, x, exp, res);
        assert!(approx_eq!(f64, exp, res, epsilon=0.00001))
    }

    #[test]
    fn test_log_with_base_3_51_of_0_286() {
        let base = 3.51;
        let x = 0.286;
        let exp = -0.99693; // https://www.symbolab.com/solver/equation-calculator/%5Clog_%7B3.51%7D%5Cleft(0.286%5Cright)?or=input
        let res = log_with_base(base, x);
        println!("base: {}, x: {}, exp: {}, res: {}", base, x, exp, res);
        assert!(approx_eq!(f64, exp, res, epsilon=0.00001))
    }
    

    #[test]
    fn test_num_tries_10() {
        let n = 10;
        let p = 1.0 / 4096.0;
        let x = probability_within_n_tries(n, p);
        println!("probability within: {} tries: {}", n, x);
        let res = num_tries_for_x_percent_chance(x, p);
        println!("result: {}", res);
        assert_eq!(n, res);
    }

    #[test]
    fn test_num_tries_163() {
        let n = 163;
        let p = 1.0 / 4096.0;
        let x = probability_within_n_tries(n, p);
        println!("probability within: {} tries: {}", n, x);
        let res = num_tries_for_x_percent_chance(x, p);
        println!("result: {}", res);
        assert_eq!(n, res);
    }

    #[test]
    fn test_num_tries_2048() {
        let n = 2048;
        let p = 1.0 / 4096.0;
        let x = probability_within_n_tries(n, p);
        println!("probability within: {} tries: {}", n, x);
        let res = num_tries_for_x_percent_chance(x, p);
        println!("result: {}", res);
        assert_eq!(n, res);
    }

    #[test]
    fn test_num_tries_4096() {
        let n = 4096;
        let p = 1.0 / 4096.0;
        let x = probability_within_n_tries(n, p);
        println!("probability within: {} tries: {}", n, x);
        let res = num_tries_for_x_percent_chance(x, p);
        println!("result: {}", res);
        assert_eq!(n, res);
    }
}