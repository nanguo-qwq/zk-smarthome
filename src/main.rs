use std::collections::HashMap;
use Zk_smarthome::zk::gateway::Gateway;
use Zk_smarthome::zk::ra::RA;
use Zk_smarthome::zk::userdevice::UserDevice;

fn main() {
    // 初始化
    let mut ra = RA::new("GW1".to_string());
    ra.initialize();

    let mut gateway = Gateway::new("GW1".to_string(), ra.n, ra.g);

    let mut user = UserDevice::new("user1".to_string());

    // 用户注册
    if let Err(e) = user.register("password123", &mut ra) {
        println!("注册失败: {}", e);
        return;
    }

    // 从 RA 获取参数
    let (n, g, gateway_id) = ra.get_parameters().expect("获取参数失败");

    // 注册用户到网关
    gateway.register_user("user1".to_string(), ra.compute_v2(user.v1));

    // 用户登录验证
    let is_authenticated = user.login("password123");
    println!("用户登录验证: {}", if is_authenticated { "成功" } else { "失败" });

    // 用户-网关认证阶段
    if is_authenticated {
        // 开始认证
        let (t1, n1) = user.start_authentication(&gateway);

        // 网关处理并发送 n2
        let n2 = rand::random::<u64>() % n;

        // 用户完成认证
        let t2 = user.complete_authentication(n2, n1);

        // 网关验证
        let authentication_result = gateway.authenticate_user("user1", t1, n2, t2);
        println!("用户-网关认证: {}", if authentication_result { "成功" } else { "失败" });

        // 密钥更新
        if authentication_result && is_authenticated{
            user.update_password("new_password", &mut gateway);
            println!("用户密钥更新: 成功");
        }
    }
}