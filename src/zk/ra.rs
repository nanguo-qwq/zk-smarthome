use std::collections::HashMap;
use rand::{random, Rng};
use sha2::{Sha256, Digest};

pub struct RA {
    users: HashMap<String, u64>, //存储用户ID和对应的V1
    pub n: u64,
    pub g: u64,
    gateway_map: HashMap<String, (String, u64, u64)>, //网关ID、IDg、Rg
}

impl RA {

    pub fn new() -> Self {
        RA {
            users: HashMap::new(),
            n: 0,
            g: 0,
            gateway_map: HashMap::new(), //网关ID、IDg、Rg
        }
    }

    //初始化
    pub fn initialize(&mut self) {
        //生成大素数 n
        self.n = self.generate_large_prime();
        //self.n=23;
        //生成生成元 g
        self.g = self.generate_generator(self.n);
        //self.g=5;
    }

    //网关注册
    pub fn register_gateway(&mut self, gid: String, idg: String, cg: u64, rg: u64) {
        self.gateway_map.insert(gid, (idg, cg, rg));
    }

    //用户注册
    pub fn receive_v1(&mut self, user_id: String, v1: u64) -> Result<(), &'static str> {
        if self.users.contains_key(&user_id) {
            return Err("用户已存在");
        }

        self.users.insert(user_id.clone(), v1);

        //V2=g^V1 mod n
        let v2 = self.compute_v2(v1);

        //发送参数
        Ok(())
    }

    pub fn get_parameters(&self, gateway_gid: &str) -> Result<(u64, u64, String, u64, u64), &'static str> {
        if self.n == 0 || self.g == 0 {
            return Err("参数未初始化");
        }

        if let Some((idg, cg, rg)) = self.gateway_map.get(gateway_gid) {
            //生成临时ID PIDu
            let pidu = format!("{}", random::<u64>());

            //X=H(IDg||Rg)
            let mut hasher = Sha256::new();
            hasher.update(idg.as_bytes());
            hasher.update(rg.to_ne_bytes());
            let x = hasher.finalize();
            let x_value = u64::from_be_bytes(x[..8].try_into().unwrap());
            //println!("1+{}",idg);

            Ok((self.n, self.g, pidu, *cg, x_value))
        } else {
            Err("网关未注册")
        }
    }

    //V2
    pub fn compute_v2(&self, v1: u64) -> u64 {
        self.mod_pow(self.g, v1, self.n)
    }

    //大素数
    fn generate_large_prime(&mut self) -> u64 {
        let mut rng = rand::thread_rng();
        loop {
            let candidate = rng.gen_range(2u64.pow(31)..2u64.pow(32));
            if self.is_prime(candidate) {
                return candidate;
            }
        }
    }

    //生成元
    fn generate_generator(&self, prime: u64) -> u64 {
        let mut rng = rand::thread_rng();
        let mut candidate;

        while {
            candidate = rng.gen_range(2..prime-1);
            !self.is_generator(candidate, prime)
        } {}

        candidate
    }

    //素数检查
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

    //生成元检查
    fn is_generator(&self, g: u64, p: u64) -> bool {
        let mut factors = Vec::new();
        let mut n = p - 1;

        //分解 p-1
        for i in 2..n {
            if n % i == 0 {
                factors.push(i);
                while n % i == 0 {
                    n /= i;
                }
            }
        }

        //检查g是否为生成元
        for &q in &factors {
            if self.mod_pow(g, (p-1)/q, p) == 1 {
                return false;
            }
        }

        true
    }

    //模幂运算
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