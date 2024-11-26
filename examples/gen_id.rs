use rand::Rng;
use verhoeff::Verhoeff;

pub const ACCOUNT_ID_LEN: usize = 20;

#[allow(dead_code)]
const ACCOUNT_ID_MAX: u64 = 10u64.pow(ACCOUNT_ID_LEN as u32 - 1) - 1;

fn main() {
    let id = rand::thread_rng().gen_range(0..=i64::MAX as u64);
    let without_check_digit = format!("{:019}", id);
    let check_digit = without_check_digit.calculate_verhoeff_check_digit();
    println!("{without_check_digit}{check_digit}");
}
