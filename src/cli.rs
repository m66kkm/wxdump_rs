use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 获取微信基址偏移
    Bias {
        /// 手机号
        #[arg(long, required = true)]
        mobile: String,

        /// 微信昵称
        #[arg(long, required = true)]
        name: String,

        /// 微信账号
        #[arg(long, required = true)]
        account: String,

        /// (可选)密钥
        #[arg(long)]
        key: Option<String>,

        /// (可选)已登录账号的微信文件夹路径
        #[arg(long)]
        db_path: Option<PathBuf>,

        /// (可选)微信版本偏移文件路径,如有，则自动更新
        #[arg(long)]
        wx_offs_path: Option<PathBuf>,
    },

    /// 获取微信信息
    Info {
        /// (可选)微信版本偏移文件路径
        #[arg(short, long)]
        wx_offs_path: Option<PathBuf>,

        /// (可选)保存路径【json文件】
        #[arg(short, long)]
        save_path: Option<PathBuf>,
    },

    /// 获取微信文件夹路径
    WxPath {
        /// (可选)需要的数据库名称(eg: -r MediaMSG;MicroMsg;FTSMSG;MSG;Sns;Emotion )
        #[arg(short = 'r', long)]
        db_types: Option<String>,

        /// (可选)'WeChat Files'路径
        #[arg(short = 'w', long)]
        wx_files: Option<PathBuf>,

        /// (可选)wxid_,用于确认用户文件夹
        #[arg(short, long)]
        wxid: Option<String>,
    },

    /// 解密微信数据库
    Decrypt {
        /// 密钥
        #[arg(short, long, required = true)]
        key: String,

        /// 数据库路径(目录or文件)
        #[arg(short, long, required = true)]
        db_path: PathBuf,

        /// 输出路径(必须是目录)[默认为当前路径下decrypted文件夹]
        #[arg(short, long, default_value = "decrypted")]
        out_path: PathBuf,
    },

    /// [测试功能]合并微信数据库(MSG.db or MediaMSG.db)
    Merge {
        /// 数据库路径(文件路径，使用英文[,]分割)
        #[arg(short, long, required = true)]
        db_path: String,

        /// 输出路径(目录或文件名)[默认为当前路径下decrypted文件夹下merge_***.db]
        #[arg(short, long, default_value = "decrypted")]
        out_path: PathBuf,
    },

    /// 聊天记录查看
    DbShow {
        /// 解密并合并后的 merge_all.db 的路径
        #[arg(long, required = true)]
        merge_path: PathBuf,

        /// (可选)微信文件夹的路径（用于显示图片）
        #[arg(long)]
        wx_path: Option<PathBuf>,

        /// (可选)微信账号(本人微信id)
        #[arg(long, default_value = "")]
        my_wxid: String,

        /// (可选)是否在线查看(局域网查看)
        #[arg(long, default_value_t = false)]
        online: bool,
    },

    /// 启动UI界面
    Ui {
        /// (可选)端口号
        #[arg(short, long, default_value_t = 5000)]
        port: u16,

        /// (可选)是否在线查看(局域网查看)
        #[arg(long, default_value_t = false)]
        online: bool,

        /// (可选)是否开启debug模式
        #[arg(long, default_value_t = false)]
        debug: bool,

        /// (可选)用于禁用自动打开浏览器
        #[arg(long = "noOpenBrowser", default_value_t = true)]
        is_open_browser: bool,
    },

    /// 启动api，不打开浏览器
    Api {
        /// (可选)端口号
        #[arg(short, long, default_value_t = 5000)]
        port: u16,

        /// (可选)是否在线查看(局域网查看)
        #[arg(long, default_value_t = false)]
        online: bool,

        /// (可选)是否开启debug模式
        #[arg(long, default_value_t = false)]
        debug: bool,
    },
}
