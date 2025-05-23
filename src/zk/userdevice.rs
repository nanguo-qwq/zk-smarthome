use sha2::{Sha256, Digest};
use crate::zk::gateway::Gateway;
use crate::zk::{fuzzyextractor, gateway, ra};
use crate::zk::fuzzyextractor::FuzzyExtractor;

pub struct UserDevice {
    id: String,
    pub v1: u64,    //H(IDu || PWu)
    n: u64,
    g: u64,
    gateway_id: String,
    pub cg: u64,
    pub pidu: String,
    x: u64,
    hu: Vec<u8> ,
}

impl UserDevice {
    pub fn new(id: String) -> Self {
        UserDevice {
            id: id.clone(),
            v1: 0,
            n: 0,
            g: 0,
            gateway_id: String::new(),
            cg: 0,
            pidu: String::new(),
            x: 0,
            hu: Vec::new(),
        }
    }

    //用户注册
    pub fn register(&mut self, pw: &str, bio: &[u8], ra: &mut ra::RA, gateway: &mut gateway::Gateway) -> Result<(), &'static str> {
        //网关准备阶段
        let (gid, cg, rg) = gateway.register_preparation();

        //模糊提取器
        let fuzzyextractor = FuzzyExtractor::new();
        let (bu, hu) = fuzzyextractor.generate(bio);
        self.hu = hu.clone();

        //V1=H(IDu||PWu||Bu)
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(pw.as_bytes());
        hasher.update(&bu);
        let hash = hasher.finalize();
        self.v1 = u64::from_be_bytes(hash[..8].try_into().unwrap());

        //发送V1给RA
        ra.receive_v1(self.id.clone(), self.v1)?;

        //RA处理后返回参数
        let (n, g, pidu, cg,x ) = ra.get_parameters(&gid)?;
        self.n = n;
        self.g = g;
        self.gateway_id = gid;
        self.cg = cg;
        self.pidu = pidu;
        self.x = x;

        //向网关注册
        gateway.register_user(self.pidu.clone(), ra.compute_v2(self.v1), self.cg);

        Ok(())
    }

    //本地登录验证
    pub fn login(&self, pw: &str, bio: &[u8]) -> bool {
        //再生算法
        let fuzzyextractor = FuzzyExtractor::new();
        let bu = fuzzyextractor.reproduce(bio, &self.hu);

        //计算V1'=H(IDu||PWu||Bu)
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(pw.as_bytes());
        hasher.update(&bu);
        let hash = hasher.finalize();
        let v1_prime = u64::from_be_bytes(hash[..8].try_into().unwrap());

        self.v1 == v1_prime
    }

    //验证网关身份
    pub fn verify_gateway(&self, x1: u64) -> bool {
        //计算H(X||PIDu)
        let mut hasher = Sha256::new();
        hasher.update(self.x.to_ne_bytes());
        hasher.update(self.pidu.as_bytes());
        let hash = hasher.finalize();
        let computed_x1 = u64::from_be_bytes(hash[..8].try_into().unwrap());

        computed_x1 == x1
    }

    //认证
    pub fn start_authentication(&self, gateway: &gateway::Gateway) -> (String, u64, u64) {
        let n = gateway.get_n();

        //生成随机数n1
        let n1 = rand::random::<u64>() % n;

        let t1 = self.mod_pow(self.g, n1, self.n);

        (self.pidu.clone(), n1, t1)
    }

    //处理响应+完成认证
    pub fn complete_authentication(&mut self, gateway: &mut gateway::Gateway, old_pidu: String, new_pidu: String, n2: u64, n1: u64, t1: u64) -> bool {
        //计算 t2
        let t2 = n1 + n2 * (self.v1 % (self.n-1));

        //发送认证结果
        let authentication_result = gateway.authenticate_user(&old_pidu, new_pidu.clone(), t1, n2, t2);

        if authentication_result {
            self.pidu = new_pidu;
        }

        authentication_result
    }

    //更新用户密钥
    pub fn update_password(&mut self, new_pw: &str,new_bio: &[u8], gateway: &mut Gateway) {
        //生成新的Bu和hu
        let fuzzyextractor = FuzzyExtractor::new();
        let (new_bu, new_hu) = fuzzyextractor.generate(new_bio);
        self.hu = new_hu.clone();

        //新V1'
        let mut hasher = Sha256::new();
        hasher.update(self.id.as_bytes());
        hasher.update(new_pw.as_bytes());
        hasher.update(&new_bu);
        let hash = hasher.finalize();
        let new_v1 = u64::from_be_bytes(hash[..8].try_into().unwrap());

        let r = rand::random::<u64>() % self.n;

        self.v1 = new_v1;

        //新V2'=g^V1'mod n
        let new_v2 = self.compute_v2() * r % self.n;

        //发送给网关
        gateway.update_user_key(self.id.clone(), new_v2, r);
    }


    //V2
    fn compute_v2(&self) -> u64 {
        self.mod_pow(self.g, self.v1, self.n)
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

