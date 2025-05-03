use std::collections::HashMap;
use sha2::{Sha256, Digest};
use crate::zk::gateway::Gateway;
use crate::zk::ra;

pub struct UserDevice {
    id: String,
    pub v1: u64,          // H(IDu || PWu)
    n: u64,
    g: u64,
    gateway_id: String,
    gateway_n: u64,
    gateway_g: u64,
}

impl UserDevice {
    pub fn new(id: String) -> Self {
        UserDevice {
            id: id.clone(),
            v1: 0,
            n: 0,
            g: 0,
            gateway_id: String::new(),
            gateway_n: 0,
            gateway_g: 0,
        }
    }

    // 注册
    pub fn register(&mut self, pw: &str, ra: &mut ra::RA) -> Result<(), &'static str> {
        // 计算 V1 = H(IDu || PWu)
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(pw.as_bytes());
        let hash = hasher.finalize();
        self.v1 = u64::from_be_bytes(hash[..8].try_into().unwrap());

        // 将 V1 发送给 RA
        ra.receive_v1(self.id.clone(), self.v1)?;

        // 从 RA 获取参数
        let (n, g, gateway_id) = ra.get_parameters()?;
        self.n = n;
        self.g = g;
        self.gateway_id = gateway_id;
        self.gateway_n = n;
        self.gateway_g = g;

        Ok(())
    }

    // 本地登录验证
    pub fn login(&self, pw: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(pw.as_bytes());
        let hash = hasher.finalize();
        let v1_prime = u64::from_be_bytes(hash[..8].try_into().unwrap());

        self.v1 == v1_prime
    }

    // 认证
    pub fn start_authentication(&self, gateway: &Gateway) -> (u64, u64) {

        let n = gateway.get_n();

        // 生成随机数 n1
        let n1 = rand::random::<u64>() % n;

        // 计算 t1 = g^n1 mod n
        let t1 = self.compute_t1(n1);

        (t1, n1)
    }

    // 处理网关响应并完成认证
    pub fn complete_authentication(&self, n2: u64, n1: u64) -> u64 {
        // 计算 t2 = n1 + n2 * V1

        let t2 = n1 + n2 * (self.v1 % (self.n-1));
        t2
    }

    // 更新用户密钥
    pub fn update_password(&mut self, new_pw: &str, gateway: &mut Gateway) {
        // 计算新的 V1'
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(new_pw.as_bytes());
        let hash = hasher.finalize();
        let new_v1 = u64::from_be_bytes(hash[..8].try_into().unwrap());

        self.v1 = new_v1;

        // 计算新的 V2' = g^V1' mod n
        let new_v2 = self.compute_v2();

        // 发送给网关
        gateway.update_user_key(self.id.clone(), new_v2);
    }

    // 辅助函数：计算 V2
    fn compute_v2(&self) -> u64 {
        self.mod_pow(self.g, self.v1, self.gateway_n)
    }

    // 辅助函数：计算 t1
    fn compute_t1(&self, n1: u64) -> u64 {
        self.mod_pow(self.gateway_g, n1, self.gateway_n)
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
