// pub mod de;
#[macro_use]
pub mod de;
pub mod error;
mod token;
pub mod tokenizer;

pub fn decompress_rl_elist(nums: Vec<i32>) -> Vec<i32> {
    let mut ans: Vec<i32> = Vec::new();
    let mut i = 0;
    while 2 * i + 1 < nums.len() {
        let num = nums[2 * i + 1];
        let mut cnt = nums[2 * i];
        while cnt > 0 {
            ans.push(num);
            cnt -= 1;
        }
        i += 1;
    }
    ans
}
