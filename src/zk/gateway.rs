use std::collections::HashMap;
use rand::Rng;
use sha2::{Digest, Sha256};

pub struct Gateway {
    id: String,
    rid: String,
    pub n: u64,
    g: u64,
    v2: u64,
    users: HashMap<String, u64>, // 存储用户ID和对应的V2
    puf_response_map: HashMap<u64, u64>, // 挑战到PUF响应的映射
    temp_id_map: HashMap<String, String>, // 旧临时ID到新临时ID的映射
}


impl Gateway {
    pub fn new(id: String, rid: String, n: u64, g: u64) -> Self {
        Gateway {
            id,
            rid,
            n,
            g,
            v2: 0,
            users: HashMap::new(),
            puf_response_map: HashMap::new(),
            temp_id_map: HashMap::new(),
        }
    }

    //网关初始化
    pub fn register_preparation(&mut self) -> (String, u64, u64) {
        // 生成随机挑战Cg
        let mut rng = rand::thread_rng();
        let cg: u64 = rng.gen();

        // 获取PUF响应Rg（这里用随机数模拟PUF响应）
        let rg: u64 = rng.gen();
        self.puf_response_map.insert(cg, rg);

        (self.id.clone(), cg, rg)
    }

    // 注册
    pub fn register_user(&mut self, pidu: String, v2: u64, cg: u64) {
        // self.users.insert(pidu, v2);
        self.users.insert(pidu, v2);
        self.v2 = v2;
    }

    // 认证

    // 处理用户认证请求 - 验证网关身份
    pub fn authenticate_gateway(&self, pidu: &str, cg: u64) -> u64 {
        if let Some(stored_v2) = self.users.get(pidu) {
            if *stored_v2 == self.v2 {
                // 获取PUF响应Rg
                if let Some(rg) = self.puf_response_map.get(&cg) {
                    // 计算X1 = H(H(IDg||Rg)||PIDu)
                    let mut hasher = Sha256::new();
                    hasher.update(self.rid.as_bytes());
                    hasher.update(rg.to_ne_bytes());
                    let hash1 = hasher.finalize();
                    let hash1_value = u64::from_be_bytes(hash1[..8].try_into().unwrap());
                    //println!("2+{}",self.rid);
                    let mut hasher2 = Sha256::new();
                    hasher2.update(hash1_value.to_ne_bytes());
                    hasher2.update(pidu.as_bytes());
                    let x1 = hasher2.finalize();

                    u64::from_be_bytes(x1[..8].try_into().unwrap())
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    }

    pub fn authenticate_user(&mut self, old_pidu: &str, new_pidu: String, t1: u64, n2: u64, t2: u64) -> bool {
        //if let Some(&v2) = self.users.get(user_id) {
        if let Some(&v2) = self.users.get(old_pidu) {
            // 更新临时ID
            self.temp_id_map.insert(old_pidu.to_string(), new_pidu.clone());
            self.users.insert(new_pidu.clone(), v2);

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