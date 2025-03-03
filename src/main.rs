mod cli;
mod wx_core;
mod db;
mod api;

use clap::Parser;
use cli::{Cli, Commands};
use colored::*;
use log::{info, error};

const WXDUMP_ASCII: &str = r"
 ██╗    ██╗██╗  ██╗██████╗ ██╗   ██╗███╗   ███╗██████╗
 ██║    ██║╚██╗██╔╝██╔══██╗██║   ██║████╗ ████║██╔══██╗
 ██║ █╗ ██║ ╚███╔╝ ██║  ██║██║   ██║██╔████╔██║██████╔╝
 ██║███╗██║ ██╔██╗ ██║  ██║██║   ██║██║╚██╔╝██║██╔═══╝
 ╚███╔███╔╝██╔╝ ██╗██████╔╝╚██████╔╝██║ ╚═╝ ██║██║
  ╚══╝╚══╝ ╚═╝  ╚═╝╚═════╝  ╚═════╝ ╚═╝     ╚═╝╚═╝
";

fn main() {
    env_logger::init();
    
    let cli = Cli::parse();
    
    // Print the banner
    println!("{}", WXDUMP_ASCII.cyan());
    println!("{}", format!(" WxDump_RS v{} ", env!("CARGO_PKG_VERSION")).cyan().on_black());
    println!("WxDump_RS功能：获取账号信息、解密数据库、查看聊天记录、导出聊天记录为html等");
    println!("{}", " options ".cyan().on_black());
    
    match cli.command {
        Commands::Bias { mobile, name, account, key, db_path, wx_offs_path } => {
            info!("Running bias command");
            match wx_core::bias_addr::run_bias_addr(account, mobile, name, key, db_path, wx_offs_path) {
                Ok(result) => println!("{:?}", result),
                Err(e) => error!("Error: {}", e),
            }
        },
        Commands::Info { wx_offs_path, save_path } => {
            info!("Running info command");
            match wx_core::wx_info::get_wx_info(&wx_offs_path, true, save_path) {
                Ok(result) => println!("{:?}", result),
                Err(e) => error!("Error: {}", e),
            }
        },
        Commands::WxPath { db_types, wx_files, wxid } => {
            info!("Running wx_path command");
            match wx_core::wx_info::get_wx_db(wx_files, db_types, wxid) {
                Ok(result) => {
                    for path in result {
                        println!("{:?}", path);
                    }
                },
                Err(e) => error!("Error: {}", e),
            }
        },
        Commands::Decrypt { key, db_path, out_path } => {
            info!("Running decrypt command");
            match wx_core::decryption::batch_decrypt(&key, &db_path, &out_path, true) {
                Ok(result) => println!("{:?}", result),
                Err(e) => error!("Error: {}", e),
            }
        },
        Commands::Merge { db_path, out_path } => {
            info!("Running merge command");
            match wx_core::merge_db::merge_db(&db_path, &out_path) {
                Ok(result) => println!("Merged database: {}", result.display()),
                Err(e) => error!("Error: {}", e),
            }
        },
        Commands::DbShow { merge_path, wx_path, my_wxid, online } => {
            info!("Running dbshow command");
            match api::start_server(Some(merge_path), wx_path, Some(my_wxid), online, 5000, false, false) {
                Ok(_) => {},
                Err(e) => error!("Error: {}", e),
            }
        },
        Commands::Ui { port, online, debug, is_open_browser } => {
            info!("Running ui command");
            match api::start_server(None, None, None, online, port, debug, is_open_browser) {
                Ok(_) => {},
                Err(e) => error!("Error: {}", e),
            }
        },
        Commands::Api { port, online, debug } => {
            info!("Running api command");
            match api::start_server(None, None, None, online, port, debug, false) {
                Ok(_) => {},
                Err(e) => error!("Error: {}", e),
            }
        },
    }
    
    println!("{}", " options ".cyan().on_black());
    println!("更多详情请查看: https://github.com/xaoyaoo/PyWxDump");
    println!("{}", format!(" WxDump_RS v{} ", env!("CARGO_PKG_VERSION")).cyan().on_black());
}
