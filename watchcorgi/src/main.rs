use chrono::prelude::*;
use clap::Parser;
use reqwest;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;
use systemstat::{saturating_sub_bytes, Platform, System};

/// Simple program to get server infomation
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Server class, GPU server of CPU server
    #[clap(short, long, value_parser)]
    node_type: String,

    /// Host server IP address, local or not_local
    #[clap(short, long, value_parser)]
    target_addr: String,
}

struct ServerInfo<'a> {
    passwd: &'a str,
    local_url: &'a str,
    non_local_url: &'a str,
}

impl<'a> ServerInfo<'a> {
    fn construct() -> Option<ServerInfo<'a>> {
        let si = ServerInfo {
            passwd: "123456",
            local_url: "http://192.168.1.206:8000/update",
            non_local_url: "http://222.19.236.158:7070/update",
        };
        Some(si)
    }
}

fn now_time() -> Option<String> {
    let local: DateTime<Local> = Local::now();
    let local_str = local.format("%Y-%m-%d %H:%M:%S").to_string();
    Some(local_str)
}

fn _system_pwdx(pid: String) -> Option<String> {
    let pwdx_output = Command::new("pwdx")
        .arg(pid)
        .output()
        .expect("failed to execute process");
    let pwdx_str = String::from_utf8_lossy(&pwdx_output.stdout);
    let pwdx_string = pwdx_str.to_string();
    Some(pwdx_string.trim().to_string())
}

fn gpu_info() -> Option<Vec<String>> {
    let nvidia_smi_output = Command::new("nvidia-smi")
        .output()
        .expect("failed to execute process");
    // println!("status: {}", output.status);
    // println!("nvidia-smi output: {}", String::from_utf8_lossy(&nvidia_smi_output.stdout));
    let nv_str = String::from_utf8_lossy(&nvidia_smi_output.stdout);
    let mut nv_split = nv_str.split("=====|");
    let mut nv_vec: Vec<&str> = nv_split.collect();
    let mut info = nv_vec[nv_vec.len() - 1];
    nv_split = info.split("+");
    nv_vec = nv_split.collect();
    info = nv_vec[0];
    nv_split = info.split("|");
    nv_vec = nv_split.collect();
    // let mut info_list: Vec<String> = Vec::new();
    let mut cwd_list: Vec<String> = Vec::new();
    for nc in nv_vec {
        let nct_0 = nc.trim();
        if nct_0.len() > 0 {
            // info_list.push(nct.to_string());
            if nct_0 != "No running processes found" {
                let nct_0_string = nct_0.to_string();
                let nct_0_split = nct_0_string.split("N/A");
                let nct_0_vec: Vec<&str> = nct_0_split.collect();
                let nct_1 = nct_0_vec[nct_0_vec.len() - 1];
                let nct_1_string = nct_1.to_string();
                let nct_1_split = nct_1_string.split("C");
                let nct_1_vec: Vec<&str> = nct_1_split.collect();
                let nct_2 = nct_1_vec[0];
                let pid = nct_2.trim().to_string();
                let pwdx = _system_pwdx(pid).unwrap();
                cwd_list.push(pwdx);
            } else {
                cwd_list.push("null".to_string());
            }
        }
    }
    // (Some(info_list), Some(cwd_list))
    Some(cwd_list)
}

fn hostname() -> Option<String> {
    let hostname_output = Command::new("hostname")
        .output()
        .expect("failed to execute process");
    // println!("{}", hostname_output.status);
    let hn_str = String::from_utf8_lossy(&hostname_output.stdout);
    let info = hn_str.trim();
    Some(info.to_string())
}

fn net_info() -> Option<HashMap<String, String>> {
    let sys = System::new();
    let mut net_info_hm: HashMap<String, String> = HashMap::new();
    match sys.networks() {
        Ok(netifs) => {
            for netif in netifs.values() {
                if netif.addrs.len() > 0 {
                    let addrs = format!("{:?}", netif.addrs[0].addr);
                    // println!("{:?}", netif.addrs);
                    let addrs_strip_1 = match addrs.strip_prefix("V4(") {
                        Some(a) => a,
                        _ => match addrs.strip_prefix("V6(") {
                            Some(b) => b,
                            _ => "null",
                        },
                    };
                    let addrs_strip_2 = addrs_strip_1.strip_suffix(")").unwrap();
                    net_info_hm.insert(netif.name.to_string(), addrs_strip_2.to_string());
                } else {
                    net_info_hm.insert(netif.name.to_string(), "null".to_string());
                }
                // println!("{} {:?}", netif.name, netif.addrs);
            }
        }
        Err(x) => println!("net_info error: {}", x),
    }
    Some(net_info_hm)
}

fn mem_info() -> Option<HashMap<String, String>> {
    let sys = System::new();
    let mut mem_info_hm: HashMap<String, String> = HashMap::new();
    match sys.memory() {
        Ok(mem) => {
            let used_str = format!("{}", saturating_sub_bytes(mem.total, mem.free));
            let total_str = format!("{}", mem.total);
            mem_info_hm.insert("used".to_string(), used_str);
            mem_info_hm.insert("total".to_string(), total_str);
        }
        Err(x) => println!("mem_info error: {}", x),
    }
    Some(mem_info_hm)
}

fn swap_info() -> Option<HashMap<String, String>> {
    let sys = System::new();
    let mut swap_info_hm: HashMap<String, String> = HashMap::new();
    match sys.swap() {
        Ok(swap) => {
            let used_str = format!("{}", saturating_sub_bytes(swap.total, swap.free));
            let total_str = format!("{}", swap.total);
            swap_info_hm.insert("used".to_string(), used_str);
            swap_info_hm.insert("total".to_string(), total_str);
        }
        Err(x) => println!("\nSwap: error: {}", x),
    }
    Some(swap_info_hm)
}

fn cpu_info() -> Option<HashMap<String, f32>> {
    let sys = System::new();
    let mut cpu_info_hm: HashMap<String, f32> = HashMap::new();
    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            thread::sleep(Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            cpu_info_hm.insert("user".to_string(), cpu.user);
            cpu_info_hm.insert("nice".to_string(), cpu.nice);
            cpu_info_hm.insert("system".to_string(), cpu.system);
            cpu_info_hm.insert("interrupt".to_string(), cpu.interrupt);
            cpu_info_hm.insert("idle".to_string(), cpu.idle);
        }
        Err(x) => println!("cpu_info error: {}", x),
    }

    match sys.cpu_temp() {
        Ok(cpu_temp) => {
            cpu_info_hm.insert("temp".to_string(), cpu_temp);
        }
        Err(x) => println!("cpu_info error: {}", x),
    }
    Some(cpu_info_hm)
}

fn _convert_sec_to_str(input: u64) -> Option<String> {
    let float_day = input as f64 / 86400.0;
    let float_hour = input as f64 / 3600.0;
    let float_min = input as f64 / 60.0;
    /*
    println!("{}", float_day);
    println!("{}", float_hour);
    println!("{}", float_min);
    */
    let day = float_day as u64;
    let hour = float_hour as u64 - day * 24;
    let min = float_min as u64 - day * 24 * 60 - hour * 60;
    let sec = input - day * 24 * 60 * 60 - hour * 60 * 60 - min * 60;
    let uptime_str = format!("{} day {} hour {} minutes {} sec", day, hour, min, sec);
    Some(uptime_str)
}

fn others_info() -> Option<HashMap<String, String>> {
    let sys = System::new();
    let mut others_info_hm: HashMap<String, String> = HashMap::new();
    match sys.uptime() {
        Ok(uptime) => {
            let uptime_sec = uptime.as_secs();
            let uptime_info = _convert_sec_to_str(uptime_sec).unwrap();
            //println!("{}", uptime_info);
            others_info_hm.insert("uptime".to_string(), uptime_info);
        }
        Err(x) => println!("uptime error: {}", x),
    }

    others_info_hm.insert("nowtime".to_string(), now_time().unwrap());
    /*
    match sys.boot_time() {
        Ok(boot_time) => {
            let boottime_info = format!("{}", boot_time);
            others_info_hm.insert("boottime".to_string(), boottime_info);
        }
        Err(x) => println!("boottime error: {}", x),
    }
    */
    Some(others_info_hm)
}

#[cfg(test)]
mod tests {
    use crate::cpu_info;
    use crate::gpu_info;
    use crate::hostname;
    use crate::mem_info;
    use crate::net_info;
    use crate::now_time;
    use crate::others_info;
    use crate::swap_info;
    use crate::ServerInfo;
    #[test]
    fn info_test() {
        let now_time_str = now_time().unwrap();
        println!("now time: {}", now_time_str);
        let server_info = ServerInfo::construct().unwrap();
        println!(
            "passwd: {}, local_url: {}, non_local_url: {}",
            server_info.passwd, server_info.local_url, server_info.non_local_url
        );
        let gpu_info_vec = gpu_info();
        println!("{:?}", gpu_info_vec.unwrap());
        let hostname = hostname().unwrap();
        println!("{}", hostname);
        // systemstat_example::test_output();
        let net_info_hm = net_info();
        println!("{:?}", net_info_hm.unwrap());
        let mem_info_hm = mem_info();
        println!("{:?}", mem_info_hm.unwrap());
        let swap_info_hm = swap_info();
        println!("{:?}", swap_info_hm.unwrap());
        let cpu_info_hm = cpu_info();
        println!("{:?}", cpu_info_hm.unwrap());
        let other_info_hm = others_info();
        println!("{:?}", other_info_hm.unwrap());
        assert_eq!(2, 1 + 1);
    }
}

fn main() {
    if cfg!(target_os = "linux") {
        let args = Args::parse();
        let server_info = ServerInfo::construct().unwrap();
        loop {
            let one_sec = Duration::from_secs(1);
            thread::sleep(one_sec);
            let gpu_flag = match args.node_type.as_str() {
                "gpu" => true,
                _ => false,
            };
            let local_flag = match args.target_addr.as_str() {
                "local" => true,
                _ => false,
            };
            let hostname = hostname().unwrap();
            let net_info_result = net_info();
            let net_info_str = format!("{:?}", net_info_result.unwrap());
            let mem_info_result = mem_info();
            let mem_info_str = format!("{:?}", mem_info_result.unwrap());
            let swap_info_result = swap_info();
            let swap_info_str = format!("{:?}", swap_info_result.unwrap());
            let cpu_info_result = cpu_info();
            let cpu_info_str = format!("{:?}", cpu_info_result.unwrap());
            let other_info_result = others_info();
            let other_info_str = format!("{:?}", other_info_result.unwrap());

            let gpu_info_str = match gpu_flag {
                true => {
                    let gpu_info_result = gpu_info();
                    let gpu_info_str = format!("{:?}", gpu_info_result.unwrap());
                    gpu_info_str
                }
                _ => "null".to_string(),
            };
            let json_data = format!(
                "{{\"passwd\": {}, \"gpu\": {}, \"hostname\": \"{}\", \"net\": {}, \"mem\": {}, \"swap\": {}, \"cpu\": {}, \"other\": {}}}",
                server_info.passwd,
                gpu_info_str,
                hostname,
                net_info_str,
                mem_info_str,
                swap_info_str,
                cpu_info_str,
                other_info_str);
            println!("{}", json_data);
            let client = reqwest::blocking::Client::new();
            let res = match local_flag {
                true => {
                    // run without parameter so in local
                    let res = client
                        .post(server_info.local_url)
                        .body(json_data)
                        .header("Content-Type", "application/json")
                        .send();
                    res
                }
                _ => {
                    let client = reqwest::blocking::Client::new();
                    let res = client
                        .post(server_info.non_local_url)
                        .body(json_data)
                        .header("Content-Type", "application/json")
                        .send();
                    res
                }
            };
            // println!("{:?}", res);
            match res {
                Ok(response) => {
                    if response.status() != 200 {
                        println!("send update data error");
                    }
                }
                Err(e) => println!("{}", e),
            }
        }
    } else if cfg!(target_os = "windows") {
        panic!("not support running at windows system!");
    } else {
        panic!("unknown os type");
    }
}
