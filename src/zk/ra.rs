use std::collections::HashMap;
use rand::Rng;
use sha2::{Sha256, Digest};

pub struct RA {
    users: HashMap<String, u64>, // 存储用户ID和对应的V1
    pub n: u64,
    pub g: u64,
    gateway_id: String,
}

impl RA {
    pub fn new(gateway_id: String) -> Self {
        RA {
            users: HashMap::new(),
            n: 0,
            g: 0,
            gateway_id,
        }
    }

    // 初始化
    pub fn initialize(&mut self) {
        // 生成大素数 n
        self.n = self.generate_large_prime();
        //self.n=23;
        // 生成生成元 g
        self.g = self.generate_generator(self.n);
        //self.g=5;
    }

    //用户注册
    pub fn receive_v1(&mut self, user_id: String, v1: u64) -> Result<(), &'static str> {
        if self.users.contains_key(&user_id) {
            return Err("用户已存在");
        }

        self.users.insert(user_id.clone(), v1);

        // 计算 V2 = g^V1 mod n
        let v2 = self.compute_v2(v1);

        // 将参数发送给网关和用户
        Ok(())
    }

    // 获取参数
    pub fn get_parameters(&self) -> Result<(u64, u64, String), &'static str> {
        if self.n == 0 || self.g == 0 {
            return Err("参数未初始化");
        }

        Ok((self.n, self.g, self.gateway_id.clone()))
    }

    // 辅助函数：计算 V2
    pub fn compute_v2(&self, v1: u64) -> u64 {
        self.mod_pow(self.g, v1, self.n)
    }

    // 辅助函数：生成大素数
    fn generate_large_prime(&mut self) -> u64 {
        let mut rng = rand::thread_rng();
        loop {
            let candidate = rng.gen_range(2u64.pow(31)..2u64.pow(32));
            if self.is_prime(candidate) {
                return candidate;
            }
        }
    }

    // 辅助函数：生成生成元
    fn generate_generator(&self, prime: u64) -> u64 {
        let mut rng = rand::thread_rng();
        let mut candidate;

        while {
            candidate = rng.gen_range(2..prime-1);
            !self.is_generator(candidate, prime)
        } {}

        candidate
    }

    // 辅助函数：检查是否为素数
    fn is_prime(&self, mut n: u64) -> bool {
        if n <= 1 {
            return false;
        }

        if n <= 3 {
            return true;
        }

        if n % 2 == 0 || n % 3 == 0 {
            return false;
        }

        let mut i = 5;
        while i * i <= n {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
            i += 6;
        }

        true
    }

    // 辅助函数：检查是否为生成元
    fn is_generator(&self, g: u64, p: u64) -> bool {
        let mut factors = Vec::new();
        let mut n = p - 1;

        // 分解 p-1
        for i in 2..n {
            if n % i == 0 {
                factors.push(i);
                while n % i == 0 {
                    n /= i;
                }
            }
        }

        // 检查 g 是否为生成元
        for &q in &factors {
            if self.mod_pow(g, (p-1)/q, p) == 1 {
                return false;
            }
        }

        true
    }

    // 模幂运算
    fn mod_pow(&self, base: u64, exponent: u64, modulus: u64) -> u64 {
        let mut result = 1;
        let mut base = base % modulus;
        let mut exponent = exponent;

        while exponent > 0 {
            if exponent % 2 == 1 {
                result = (result * base) % modulus;
            }
            exponent = exponent >> 1;
            base = (base * base) % modulus;
        }

        result
    }
}