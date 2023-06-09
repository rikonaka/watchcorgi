use chrono::prelude::*;
use clap::Parser;
use reqwest;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::Duration;
use systemstat::{saturating_sub_bytes, Platform, System};

/// Simple program to get server infomation
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Server type, GPU server of CPU server
    #[clap(long, default_value = "gpu")]
    server: String,

    /// Host server IP address
    #[clap(long, default_value = "http://127.0.0.1:7070/update")]
    address: String,

    /// Upload interval (sec)
    #[clap(long, default_value_t = 60)]
    interval: u64,
}

struct ServerInfo<'a> {
    password: &'a str,
    addr: &'a str,
}

impl<'a> ServerInfo<'a> {
    fn new(password: &'a str, addr: &'a str) -> ServerInfo<'a> {
        // http://192.168.1.206:7070/update
        // http://222.19.236.142:7070/update
        ServerInfo { password, addr }
    }
}

#[derive(Serialize)]
struct GPUDetail {
    name: String,
    driver_version: String,
    temperature_gpu: String,
    utilization_gpu: String,
    utilization_memory: String,
    memory_total: String,
    memory_free: String,
    memory_used: String,
}

impl GPUDetail {
    fn empty() -> GPUDetail {
        GPUDetail {
            name: "".to_string(),
            driver_version: "".to_string(),
            temperature_gpu: "".to_string(),
            utilization_gpu: "".to_string(),
            utilization_memory: "".to_string(),
            memory_total: "".to_string(),
            memory_free: "".to_string(),
            memory_used: "".to_string(),
        }
    }
}

#[derive(Serialize)]
struct GPUInfo {
    detail: Vec<GPUDetail>,
    users: Vec<String>,
}

impl GPUInfo {
    fn empty() -> GPUInfo {
        let detail = GPUDetail {
            name: "".to_string(),
            driver_version: "".to_string(),
            temperature_gpu: "".to_string(),
            utilization_gpu: "".to_string(),
            utilization_memory: "".to_string(),
            memory_total: "".to_string(),
            memory_free: "".to_string(),
            memory_used: "".to_string(),
        };
        GPUInfo {
            detail: vec![detail],
            users: vec!["null".to_string()],
        }
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

fn _gpu_users() -> (Vec<String>, i32) {
    let gpu_users = |nv_output: String| -> Vec<String> {
        let mut gpu_users_vec: Vec<String> = Vec::new();
        let mut nv_split = nv_output.split("=====|");
        let mut nv_vec: Vec<&str> = nv_split.collect();
        let mut info = nv_vec[nv_vec.len() - 1];
        nv_split = info.split("+");
        nv_vec = nv_split.collect();
        info = nv_vec[0];
        nv_split = info.split("|");
        nv_vec = nv_split.collect();
        // let mut info_list: Vec<String> = Vec::new();
        for nc in nv_vec {
            let nct_0 = nc.trim();
            if nct_0.len() > 0 {
                // info_list.push(nct.to_string());
                if nct_0.contains("No running processes found") {
                    gpu_users_vec.push("no running processes found".to_string());
                } else {
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
                    gpu_users_vec.push(pwdx);
                }
            }
        }
        gpu_users_vec
    };

    let nvidia_smi_output = Command::new("nvidia-smi")
        .output()
        .expect("failed to execute process");
    // println!("status: {}", output.status);
    // println!("nvidia-smi output: {}", String::from_utf8_lossy(&nvidia_smi_output.stdout));
    let nv_output = String::from_utf8_lossy(&nvidia_smi_output.stdout).to_string();
    let mut status = 0; // 0 success, -1 failed
    let gpu_users_vec =
        if nv_output.contains("Driver Version:") & nv_output.contains("CUDA Version:") {
            gpu_users(nv_output)
        } else {
            status = -1;
            vec!["driver failed".to_string()]
        };
    (gpu_users_vec, status)
}

fn gpu_info() -> GPUInfo {
    // get users from nvidia-smi
    let (users, status) = _gpu_users();
    let detail = if status != -1 {
        let nvidia_smi_query_output = Command::new("nvidia-smi")
            .args(["--query-gpu=name,driver_version,temperature.gpu,utilization.gpu,utilization.memory,memory.total,memory.free,memory.used", "--format=csv,noheader"])
            .output()
            .expect("failed to execute process");
        // single gpu
        // NVIDIA GeForce RTX 3090 Ti, 530.41.03, 36, 0 %, 0 %, 24564 MiB, 24247 MiB, 0 MiB
        // multi gpu
        // NVIDIA GeForce RTX 2080 Ti, 510.47.03, 40, 0 %, 0 %, 11264 MiB, 8456 MiB, 2562 MiB
        // NVIDIA GeForce RTX 2080 Ti, 510.47.03, 48, 0 %, 0 %, 11264 MiB, 3058 MiB, 7960 MiB
        let nv_query_output = String::from_utf8_lossy(&nvidia_smi_query_output.stdout).to_string();
        let gpus_vec: Vec<&str> = nv_query_output.trim().split("\n").collect();
        let mut detail = Vec::new();
        for gpu in gpus_vec {
            let split_line: Vec<&str> = gpu.split(",").collect();
            let gpu_info = if split_line.len() != 8 {
                GPUDetail::empty()
            } else {
                let name = split_line[0].trim().to_string();
                let driver_version = split_line[1].trim().to_string();
                let temperature_gpu = split_line[2].trim().to_string();
                let utilization_gpu = if split_line[3].trim().to_string().contains("%") {
                    split_line[3].trim().to_string()
                } else {
                    "Err".to_string()
                };
                let utilization_memory = if split_line[4].trim().to_string().contains("%") {
                    split_line[4].trim().to_string()
                } else {
                    "Err".to_string()
                };
                let memory_total = if split_line[5].trim().to_string().contains("MiB") {
                    split_line[5].trim().to_string()
                } else {
                    "Err".to_string()
                };
                let memory_free = if split_line[6].trim().to_string().contains("MiB") {
                    split_line[6].trim().to_string()
                } else {
                    "Err".to_string()
                };
                let memory_used = if split_line[7].trim().to_string().contains("MiB") {
                    split_line[7].trim().to_string()
                } else {
                    "Err".to_string()
                };
                GPUDetail {
                    name,
                    driver_version,
                    temperature_gpu,
                    utilization_gpu,
                    utilization_memory,
                    memory_total,
                    memory_free,
                    memory_used,
                }
            };
            detail.push(gpu_info);
        }
        detail
    } else {
        vec![GPUDetail::empty()]
    };
    GPUInfo { detail, users }
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

fn net_info() -> HashMap<String, String> {
    let sys = System::new();
    let mut net_info_hm: HashMap<String, String> = HashMap::new();
    match sys.networks() {
        Ok(netifs) => {
            for netif in netifs.values() {
                if netif.addrs.len() > 0 {
                    let addrs = format!("{:?}", netif.addrs[0].addr);
                    // println!("{:?}", addrs);
                    if !addrs.contains("Empty") {
                        let addrs_strip_1 = match addrs.strip_prefix("V4(") {
                            Some(a) => a,
                            _ => match addrs.strip_prefix("V6(") {
                                Some(b) => b,
                                _ => "null",
                            },
                        };
                        let addrs_strip_2 = addrs_strip_1.strip_suffix(")").unwrap();
                        net_info_hm.insert(netif.name.to_string(), addrs_strip_2.to_string());
                    }
                } else {
                    net_info_hm.insert(netif.name.to_string(), "null".to_string());
                }
                // println!("{} {:?}", netif.name, netif.addrs);
            }
        }
        Err(x) => println!("net_info error: {}", x),
    }
    net_info_hm
}

fn mem_info() -> HashMap<String, String> {
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
    mem_info_hm
}

fn swap_info() -> HashMap<String, String> {
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
    swap_info_hm
}

fn cpu_info() -> HashMap<String, f32> {
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
    cpu_info_hm
}

fn _convert_sec_to_str(input: u64) -> String {
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
    uptime_str
}

fn others_info() -> HashMap<String, String> {
    let sys = System::new();
    let mut others_info_hm: HashMap<String, String> = HashMap::new();
    match sys.uptime() {
        Ok(uptime) => {
            let uptime_sec = uptime.as_secs();
            let uptime_info = _convert_sec_to_str(uptime_sec);
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
    others_info_hm
}

fn main() {
    if cfg!(target_os = "linux") {
        let args = Args::parse();
        let server_info = ServerInfo::new("123456", &args.address);
        let interval = args.interval;
        let sleep_duration = Duration::from_secs(interval);
        let gpu_flag = match args.server.as_str() {
            "gpu" => true,
            _ => false,
        };
        loop {
            let hostname = hostname().unwrap();
            let net_info_result = net_info();
            let mem_info_result = mem_info();
            let swap_info_result = swap_info();
            let cpu_info_result = cpu_info();
            let other_info_result = others_info();

            let gpu_info_result = match gpu_flag {
                true => gpu_info(),
                _ => GPUInfo::empty(),
            };
            let json_data = json!({
                "password": server_info.password,
                "gpu": gpu_info_result,
                "hostname": hostname,
                "net": net_info_result,
                "mem": mem_info_result,
                "swap": swap_info_result,
                "cpu": cpu_info_result,
                "other": other_info_result,
            });
            let client = reqwest::blocking::Client::new();
            let res = client.post(server_info.addr).json(&json_data).send();
            match res {
                Ok(response) => {
                    if response.status() != 200 {
                        println!("Send update data error: {}", response.status());
                    }
                }
                Err(e) => println!("{}", e),
            }
            thread::sleep(sleep_duration);
        }
    } else if cfg!(target_os = "windows") {
        panic!("not support running at windows system!");
    } else {
        panic!("unknown os type");
    }
}
