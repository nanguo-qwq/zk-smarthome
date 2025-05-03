use std::collections::HashMap;

pub struct Gateway {
    id: String,
    n: u64,
    g: u64,
    v2: u64,
    users: HashMap<String, u64>, // 存储用户ID和对应的V2
}

impl Gateway {
    pub fn new(id: String, n: u64, g: u64) -> Self {
        Gateway {
            id,
            n,
            g,
            v2: 0,
            users: HashMap::new(),
        }
    }

    // 注册
    pub fn register_user(&mut self, user_id: String, v2: u64) {
        self.users.insert(user_id, v2);
        self.v2 = v2;
    }

    // 认证
    pub fn authenticate_user(&self, user_id: &str, t1: u64, n2: u64, t2: u64) -> bool {
        if let Some(&v2) = self.users.get(user_id) {
            // 计算 g^t2 mod n
            let left_side = mod_pow(self.g, t2, self.n);

            // 计算 t1 * (v2^n2) mod n
            let right_side = (t1 * mod_pow(v2, n2, self.n)) % self.n;

            left_side == right_side
        } else {
            false
        }
    }


    // 密钥更新
    pub fn update_user_key(&mut self, user_id: String, new_v2: u64) {
        if self.users.contains_key(&user_id) {
            self.users.insert(user_id, new_v2);
        }
    }

    // 获取ID
    pub fn get_id(&self) -> &str {
        &self.id
    }

    // 获取n
    pub fn get_n(&self) -> u64 {
        self.n
    }

    // 获取g
    pub fn get_g(&self) -> u64 {
        self.g
    }
}

// 模幂运算
fn mod_pow(base: u64, exponent: u64, modulus: u64) -> u64 {
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