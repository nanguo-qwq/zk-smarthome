use std::collections::HashMap;
use rand::{random, Rng};
use Zk_smarthome::zk::gateway::Gateway;
use Zk_smarthome::zk::ra::RA;
use Zk_smarthome::zk::userdevice::UserDevice;

fn main() {
    // 初始化
    let mut ra = RA::new();
    ra.initialize();

    let rid="IDg1";
    let mut gateway = Gateway::new("GW1".to_string(), rid.to_string(), ra.n, ra.g);

    let mut user = UserDevice::new("user1".to_string());

    // 用户注册
    // 1. 网关准备阶段
    let (gid, cg, rg) = gateway.register_preparation();

    // 2. 网关向 RA 注册
    ra.register_gateway(gid.clone(), rid.to_string(), cg, rg);
    //println!("0+{}",gid.clone());

    // 3. 用户注册
    if let Err(e) = user.register("password123", &mut ra, &mut gateway) {
        println!("注册失败: {}", e);
        return;
    }

    // 4. 从 RA 获取参数
    let (n, g, pidu, cg, x) = ra.get_parameters(&gid).expect("获取参数失败");


    // 用户登录验证
    let is_authenticated = user.login("password123");
    println!("用户登录验证: {}", if is_authenticated { "成功" } else { "失败" });

    // 用户-网关认证阶段
    if is_authenticated {
        // 1. 验证网关身份
        let x1 = gateway.authenticate_gateway(&user.pidu, user.cg);
        let gateway_verified = user.verify_gateway(x1);
        println!("网关身份验证: {}", if gateway_verified { "成功" } else { "失败" });

        if gateway_verified {
            // 2. 用户认证
            // 开始认证
            let (pidu,n1, t1) = user.start_authentication(&gateway);

            // 网关处理并发送 PIDu' 和 n2
            let new_pidu = format!("{}", random::<u64>());
            let n2 = rand::random::<u64>() % n;

            // 用户完成认证
            let authentication_result = user.complete_authentication(&mut gateway, pidu, new_pidu.clone(), n2, n1, t1);
            println!("用户认证: {}", if authentication_result { "成功" } else { "失败" });

        }
        // 密钥更新
        user.update_password("new_password", &mut gateway);
        println!("用户密钥更新: 成功");

    }

}