use actix_cors::Cors;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::Local;
use clap::Parser;
use itertools::Itertools;
use once_cell::sync::OnceCell;
use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};
// use sqlx::types::Json;
// use sqlx::PgPool;
use std::collections::HashMap;

// static PG_CONNECTION: OnceCell<PgPool> = OnceCell::new();
static RD_CONNECTION: OnceCell<Client> = OnceCell::new();

// toy password
const PASSWORD: &str = "123456";
// const PG_DATABASE_URL: &str = "postgresql://user:password@127.0.0.1:5432/server";

/// Simple program to get server infomation
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Server listen address
    #[clap(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Server listen port
    #[clap(short, long, default_value_t = 7070)]
    port: u16,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
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

#[derive(Deserialize, Serialize, Clone)]
struct GPUInfo {
    detail: Vec<GPUDetail>,
    users: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct ServerInfo {
    password: String,
    gpu: GPUInfo,
    hostname: String,
    net: HashMap<String, String>,
    mem: HashMap<String, String>,
    swap: HashMap<String, String>,
    cpu: HashMap<String, f32>,
    other: HashMap<String, String>,
}

// async fn pg_insert_watchdog(hostname: &str, server_info: &ServerInfo) -> bool {
//     let pool = PG_CONNECTION.get().unwrap();
//     let rec =
//         match sqlx::query("INSERT INTO watchdog (hostname, data, update_time) VALUES ($1, $2, $3)")
//             .bind(hostname)
//             .bind(Json(server_info))
//             .bind(Local::now())
//             .execute(pool)
//             .await
//         {
//             Ok(_) => true,
//             Err(e) => {
//                 println!("sqlx error: {}", e);
//                 false
//             }
//         };
//
//     // println!("{:?}", rec);
//     rec
// }

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body(format!("pong"))
}

fn _redis_connection() -> Connection {
    let client = RD_CONNECTION.get().unwrap();
    let con = match client.get_connection() {
        Ok(c) => c,
        Err(e) => panic!("Get redis connection failed: {}", e),
    };
    con
}

fn _redis_get() -> HashMap<String, ServerInfo> {
    let mut con = _redis_connection();
    let keys: Vec<String> = match con.keys("*") {
        Ok(r) => r,
        Err(e) => panic!("Get all keys failed: {}", e),
    };
    let mut database: HashMap<String, ServerInfo> = HashMap::new();
    for k in keys {
        let v: String = con.get(&k).unwrap();
        let v: ServerInfo = serde_json::from_str(&v).unwrap();
        database.insert(k, v);
    }
    database
}

#[post("/update")]
async fn update(server_info: web::Json<ServerInfo>) -> impl Responder {
    let mut con = _redis_connection();
    // println!("Get: {}", gpu_info.hostname);
    if server_info.password != PASSWORD {
        HttpResponse::Ok().body(format!("Password wrong!"))
    } else {
        let hostname = &server_info.hostname;
        let serde_server_info = match serde_json::to_string(&server_info) {
            Ok(s) => s,
            Err(e) => panic!("Convert struct to string failed: {}", e),
        };
        let _: () = con
            .set_ex(hostname, serde_server_info, 60)
            .expect("Redis set failed");

        // match pg_insert_watchdog(hostname, &server_info).await {
        //     true => (),
        //     false => println!("Insert into postgre failed"),
        // }

        HttpResponse::Ok().body(format!("Welcome {}!", hostname))
    }
}

#[get("/info")]
async fn info() -> impl Responder {
    let database = _redis_get();

    let mut name_title = String::from("name");
    let mut cpu_system_title = String::from("cpu[s]");
    let mut cpu_user_title = String::from("cpu[u]");
    let mut gpu_name_title = String::from("gpu device");
    let mut gpu_util_title = String::from("gpu[u]");
    let mut gpu_memory_title = String::from("gpu[m]");
    let mut gpu_user_title = String::from("gpu user");
    let mut nowtime_title = String::from("update time");

    let mut name_len = name_title.len();
    let mut cpu_system_len = cpu_system_title.len();
    let mut cpu_user_len = cpu_user_title.len();
    let mut gpu_name_len = gpu_name_title.len();
    let mut gpu_util_len = gpu_util_title.len();
    let mut gpu_memory_len = gpu_memory_title.len();
    let mut gpu_user_len = gpu_user_title.len();
    let mut nowtime_len = nowtime_title.len();

    let mut new_database = HashMap::new();
    for (name, mut server_info) in database {
        if name.len() > name_len {
            name_len = name.len();
        }
        let gpu_detail_vec = &server_info.gpu.detail;
        for detail in gpu_detail_vec {
            let gpu_name = format!("{}({})", detail.name, detail.driver_version);
            if gpu_name.len() > gpu_name_len {
                gpu_name_len = gpu_name.len();
            }
            if detail.utilization_gpu.len() > gpu_util_len {
                gpu_util_len = detail.utilization_gpu.len();
            }
            let gpu_memory = format!("{}/{}", detail.memory_used, detail.memory_total);
            if gpu_memory.len() > gpu_memory_len {
                gpu_memory_len = gpu_memory.len();
            }
        }

        let gpu_users_vec = &server_info.gpu.users;
        let new_gpu_vec = gpu_users_vec;
        // let mut new_gpu_vec = Vec::new();
        // for gpu in gpu_users_vec {
        //     if gpu.contains("/") {
        //         let tmp_vec: Vec<&str> = gpu.split("/").collect();
        //         if tmp_vec.len() > 4 {
        //             let new_gpu = tmp_vec[4];
        //             new_gpu_vec.push(new_gpu.to_string());
        //         } else {
        //             // new_gpu_vec.push("".to_string());
        //             new_gpu_vec.push(tmp_vec[tmp_vec.len() - 1].to_string());
        //         }
        //     } else if gpu.contains("no running processes found") {
        //         new_gpu_vec.push("null".to_string());
        //     } else if gpu.contains("driver failed") {
        //         new_gpu_vec.push("driver failed".to_string());
        //     } else {
        //         // new_gpu_vec.push("".to_string());
        //         new_gpu_vec.push(gpu.to_string());
        //     }
        // }
        for gpu in new_gpu_vec {
            if gpu.len() > gpu_user_len {
                gpu_user_len = gpu.len();
            }
        }
        // server_info.gpu.users = new_gpu_vec;

        let cpu_hm = &server_info.cpu;
        let cpu_user = format!("{:.1} %", cpu_hm.get("user").unwrap() * 100.0);
        let cpu_system = format!("{:.1} %", cpu_hm.get("system").unwrap() * 100.0);
        if cpu_user.len() > cpu_user_len {
            cpu_user_len = cpu_user.len();
        }
        if cpu_system.len() > cpu_system_len {
            cpu_system_len = cpu_system.len();
        }

        let nowtime = server_info.other.get("nowtime").unwrap();
        let nowtime_vec: Vec<&str> = nowtime.split(" ").collect();
        let nowtime = if nowtime_vec.len() > 1 {
            nowtime_vec[1]
        } else {
            "null"
        };
        if nowtime.len() > nowtime_len {
            nowtime_len = nowtime.len();
        }

        new_database.insert(name, server_info);
    }

    // name
    let mut lines_cut = String::from("+");
    for _ in 0..name_len {
        lines_cut.push_str("-");
    }
    // cpu system
    lines_cut.push_str("+");
    for _ in 0..cpu_system_len {
        lines_cut.push_str("-");
    }
    // cpu user
    lines_cut.push_str("+");
    for _ in 0..cpu_user_len {
        lines_cut.push_str("-");
    }
    // gpu name
    lines_cut.push_str("+");
    for _ in 0..gpu_name_len {
        lines_cut.push_str("-");
    }
    // gpu util
    lines_cut.push_str("+");
    for _ in 0..gpu_util_len {
        lines_cut.push_str("-");
    }
    // gpu mem
    lines_cut.push_str("+");
    for _ in 0..gpu_memory_len {
        lines_cut.push_str("-");
    }
    // gpu user
    lines_cut.push_str("+");
    for _ in 0..gpu_user_len {
        lines_cut.push_str("-");
    }
    // last updated
    lines_cut.push_str("+");
    for _ in 0..nowtime_len {
        lines_cut.push_str("-");
    }
    lines_cut.push_str("+");
    lines_cut.push_str("\n");

    let mut lines = String::from("");
    for (k, s) in new_database.iter().sorted_by_key(|x| x.0) {
        let gpu_users_vec = &s.gpu.users;
        let gpu_detail_vec = &s.gpu.detail;
        let mut add = false;
        let max_row = if gpu_users_vec.len() > gpu_detail_vec.len() {
            gpu_users_vec.len()
        } else {
            gpu_detail_vec.len()
        };
        // for (i, gpu) in gpu_users_vec.iter().enumerate() {
        for i in 0..max_row {
            // gpu user
            let gpu_user = if i < gpu_users_vec.len() {
                gpu_users_vec[i].clone()
            } else {
                "".to_string()
            };
            // gpu detail
            let gpu_detail = if i < gpu_detail_vec.len() {
                gpu_detail_vec[i].clone()
            } else {
                GPUDetail::empty()
            };

            // name
            let mut name = if i == 0 {
                k.to_string()
            } else {
                String::from("")
            };
            for _ in 0..((name_len - name.len()) / 2) {
                name = format!(" {} ", name);
            }
            if name.len() != name_len {
                name = format!(" {}", name)
            }

            // cpu system
            let mut cpu_system = if i == 0 {
                format!("{:.1} %", s.cpu.get("system").unwrap() * 100.0)
            } else {
                String::from("")
            };
            for _ in 0..((cpu_system_len - cpu_system.len()) / 2) {
                cpu_system = format!(" {} ", cpu_system);
            }
            if cpu_system.len() != cpu_system_len {
                cpu_system = format!(" {}", cpu_system);
            }

            // cpu user
            let mut cpu_user = if i == 0 {
                format!("{:.1} %", s.cpu.get("user").unwrap() * 100.0)
            } else {
                String::from("")
            };
            for _ in 0..((cpu_user_len - cpu_user.len()) / 2) {
                cpu_user = format!(" {} ", cpu_user);
            }
            if cpu_user.len() != cpu_user_len {
                cpu_user = format!(" {}", cpu_user);
            }

            // gpu name
            let mut gpu_name =
                if (gpu_detail.name.len() == 0) & (gpu_detail.driver_version.len() == 0) {
                    "".to_string()
                } else {
                    format!("{}({})", gpu_detail.name, gpu_detail.driver_version)
                };
            for _ in 0..((gpu_name_len - gpu_name.len()) / 2) {
                gpu_name = format!(" {} ", gpu_name);
            }
            if gpu_name.len() != gpu_name_len {
                gpu_name = format!(" {}", gpu_name);
            }

            // gpu util
            let mut gpu_util = gpu_detail.utilization_gpu;
            for _ in 0..((gpu_util_len - gpu_util.len()) / 2) {
                gpu_util = format!(" {} ", gpu_util);
            }
            if gpu_util.len() != gpu_util_len {
                gpu_util = format!(" {}", gpu_util);
            }

            // gpu memory
            let mut gpu_memory =
                if (gpu_detail.memory_used.len() == 0) & (gpu_detail.memory_total.len() == 0) {
                    "".to_string()
                } else {
                    format!("{}/{}", gpu_detail.memory_used, gpu_detail.memory_total)
                };
            for _ in 0..((gpu_memory_len - gpu_memory.len()) / 2) {
                gpu_memory = format!(" {} ", gpu_memory);
            }
            if gpu_memory.len() != gpu_memory_len {
                gpu_memory = format!(" {}", gpu_memory);
            }

            // gpu user
            let mut gpu_users = gpu_user.to_string();
            for _ in 0..((gpu_user_len - gpu_users.len()) / 2) {
                gpu_users = format!(" {} ", gpu_users);
            }
            if gpu_users.len() != gpu_user_len {
                gpu_users = format!(" {}", gpu_users);
            }

            // nowtime
            let mut nowtime = if i == 0 {
                let nowtime = s.other.get("nowtime").unwrap();
                let nowtime_vec: Vec<&str> = nowtime.split(" ").collect();
                if nowtime_vec.len() > 1 {
                    nowtime_vec[1].to_string()
                } else {
                    "null".to_string()
                }
            } else {
                "".to_string()
            };
            for _ in 0..((nowtime_len - nowtime.len()) / 2) {
                nowtime = format!(" {} ", nowtime);
            }
            if nowtime.len() != nowtime_len {
                nowtime = format!(" {}", nowtime);
            }

            let l = format!(
                "|{}|{}|{}|{}|{}|{}|{}|{}|\n",
                name, cpu_system, cpu_user, gpu_name, gpu_util, gpu_memory, gpu_users, nowtime
            );
            lines.push_str(&l);
            add = true;
        }
        if add == true {
            lines.push_str(&lines_cut);
        }
    }

    for _ in 0..((name_len - name_title.len()) / 2) {
        name_title = format!(" {} ", name_title);
    }
    if name_title.len() != name_len {
        name_title = format!(" {}", name_title);
    }

    for _ in 0..((cpu_system_len - cpu_system_title.len()) / 2) {
        cpu_system_title = format!(" {} ", cpu_system_title);
    }
    if cpu_system_title.len() != cpu_system_len {
        cpu_system_title = format!(" {}", cpu_system_title);
    }

    for _ in 0..((cpu_user_len - cpu_user_title.len()) / 2) {
        cpu_user_title = format!(" {} ", cpu_user_title);
    }
    if cpu_user_title.len() != cpu_user_len {
        cpu_user_title = format!(" {}", cpu_user_title);
    }

    for _ in 0..((gpu_name_len - gpu_name_title.len()) / 2) {
        gpu_name_title = format!(" {} ", gpu_name_title);
    }
    if gpu_name_title.len() != gpu_name_len {
        gpu_name_title = format!(" {}", gpu_name_title);
    }

    for _ in 0..((gpu_util_len - gpu_util_title.len()) / 2) {
        gpu_util_title = format!(" {} ", gpu_util_title);
    }
    if gpu_util_title.len() != gpu_util_len {
        gpu_util_title = format!(" {}", gpu_util_title);
    }

    for _ in 0..((gpu_memory_len - gpu_memory_title.len()) / 2) {
        gpu_memory_title = format!(" {} ", gpu_memory_title);
    }
    if gpu_memory_title.len() != gpu_memory_len {
        gpu_memory_title = format!(" {}", gpu_memory_title);
    }

    for _ in 0..((gpu_user_len - gpu_user_title.len()) / 2) {
        gpu_user_title = format!(" {} ", gpu_user_title);
    }
    if gpu_user_title.len() != gpu_user_len {
        gpu_user_title = format!(" {}", gpu_user_title);
    }

    for _ in 0..((nowtime_len - nowtime_title.len()) / 2) {
        nowtime_title = format!(" {} ", nowtime_title);
    }
    if nowtime_title.len() != nowtime_len {
        nowtime_title = format!(" {}", nowtime_title);
    }

    let lines_title = format!(
        "|{}|{}|{}|{}|{}|{}|{}|{}|\n",
        name_title,
        cpu_system_title,
        cpu_user_title,
        gpu_name_title,
        gpu_util_title,
        gpu_memory_title,
        gpu_user_title,
        nowtime_title
    );
    let date_as_string = Local::now().format("%Y-%m-%d %H:%M:%S");
    let info_str = format!(">> {} [watchcorgi]\n", date_as_string);
    let powered = "Powered by Rust\n";
    let lines = format!(
        "{}{}{}{}{}{}",
        info_str, lines_cut, lines_title, lines_cut, lines, powered
    );
    HttpResponse::Ok().body(lines)
}

#[get("/info2")]
async fn info2() -> impl Responder {
    let database = _redis_get();
    HttpResponse::Ok().json(database)
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use itertools::Itertools;
    use std::collections::HashMap;
    #[test]
    fn test_hashmap() {
        // some test code here
        let mut hashmap = HashMap::new();
        hashmap.insert("a", 1);
        hashmap.insert("b", 2);
        hashmap.insert("c", 3);
        for (k, v) in hashmap.iter().sorted_by_key(|x| x.0) {
            println!("{:?}: {:?}", k, v);
        }
        let keys = hashmap.keys();
        println!("{:?}", keys);
        let mut keys_vec = Vec::new();
        for k in keys {
            keys_vec.push(k.to_string());
        }
        println!("{:?}", keys_vec);
        for x in 0..10 {
            println!("{}", x);
        }
        let x = "123";
        let x_len = 10;
        for o in 0..((x_len - x.len()) / 2) {
            println!("o: {}", o);
        }
        let date_as_string = Local::now().format("%Y-%m-%d %H:%M:%S");
        println!("{}", date_as_string);
        assert_eq!(10, 10);
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Web is running...");
    // let pool = PgPool::connect(PG_DATABASE_URL)
    //     .await
    //     .expect("Connect to postgre failed");
    // PG_CONNECTION.set(pool).expect("Set PG_CONNECTION failed");

    let args = Args::parse();

    let client = match redis::Client::open("redis://127.0.0.1/") {
        Ok(c) => c,
        Err(e) => panic!("Connect to redis failed: {}", e),
    };
    RD_CONNECTION.set(client).expect("Set RD_CONNECTION failed");

    HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin().send_wildcard();
        App::new()
            .wrap(cors)
            .service(hello)
            .service(ping)
            .service(update)
            .service(info)
            .service(info2)
    })
    .bind((args.address, args.port))?
    .run()
    .await
}
