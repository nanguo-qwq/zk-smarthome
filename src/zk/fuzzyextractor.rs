use rand::Rng;

//简单模拟模糊提取器
pub struct FuzzyExtractor {
    bio: Vec<u8>,
    bu: Vec<u8>,
    hu: Vec<u8>,
}
impl FuzzyExtractor {
    pub fn new() -> Self {
        FuzzyExtractor {
            bio: vec![],
            bu: vec![],
            hu: vec![],
        }
    }
    //模糊提取器的生成函数
    pub fn generate(&self, bio: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut rng = rand::thread_rng();
        let mut bu = vec![0u8; 16];
        for val in bu.iter_mut() {
            *val = rng.gen_range(0..=177);
        }

        let mut hu = bio.iter().zip(bu.iter()).map(|(a, b)| a + b).collect();;
        (bu, hu)
    }

    //模糊提取器的再生函数
    pub fn reproduce(&self, bio: &[u8], hu: &[u8]) -> Vec<u8> {
        let mut bu = hu.iter().zip(bio.iter()).map(|(a, b)| a - b).collect();
        bu
    }
}