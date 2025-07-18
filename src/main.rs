mod encrypt;
mod blockchain;

use clap::Parser;
use std::fs;
use std::fs::create_dir_all;
use std::path::Path;
use encrypt::encrypt_and_hash;
use blockchain::{create_block, Block};
use serde_json;
use std::ffi::OsStr;
//use std::os::windows::fs::PermissionsExt; // Windows 專用
// use std::os::unix::fs::PermissionsExt; // 若在 Unix 系統請改用這行



/// CLI 參數定義
#[derive(Parser)]
#[command(name = "DiaryChain")]
#[command(about = "將 Markdown 日記加密並上鏈，或驗證區塊鏈")]
struct Cli {
    /// Markdown 檔案路徑（上鏈模式使用）
    #[arg(short, long)]
    input: Option<String>,

    /// 驗證區塊鏈
    #[arg(long)]
    verify: bool,
	
	/// 批次處理整個 diary 資料夾
	#[arg(long)]
	batch: bool,
	
	/// 檢查某篇日記是否已上鏈
	#[arg(long)]
	check: Option<String>,
	
	/// 將區塊鏈匯出為 CSV 報告
	#[arg(long)]
	export: Option<String>,
	/// 將 chain.json 設為唯讀，防止修改
	#[arg(long)]
	lock_chain: bool,
}

fn main() {
    let cli = Cli::parse();

    // 🔍 驗證模式
    if cli.verify {
        verify_chain("output/chain.json");
        return;
    }
	
	if cli.batch {
    batch_process("diary", "output/chain.json");
    return;
}
	if let Some(filename) = cli.check {
    check_diary(&filename, "output/chain.json");
    return;
}

	if let Some(csv_path) = cli.export {
    export_chain_to_csv("output/chain.json", &csv_path);
    return;
}
	if cli.lock_chain {
    lock_chain_file("output/chain.json");
    return;
}
    // 📥 上鏈模式：取得輸入檔案路徑
    let input_path = cli.input.unwrap_or_else(|| {
        eprintln!("❌ 請提供 --input <檔案路徑>");
        std::process::exit(1);
    });

    if !Path::new(&input_path).exists() {
        eprintln!("❌ 找不到檔案：{}", input_path);
        std::process::exit(1);
    }

    let content = fs::read_to_string(&input_path)
        .expect("無法讀取指定檔案");

    // 🔐 加密與 hash
    let key = b"anexampleverysecurekey12345678!!"; // 32 bytes
    let (_ciphertext, hash) = encrypt_and_hash(&content, key);
    println!("✅ 加密完成，hash: {}", hash);

    // 📁 建立 output 資料夾
    create_dir_all("output").expect("無法建立 output 資料夾");

    // ⛓️ 讀取現有鏈
    let chain_path = "output/chain.json";
    let mut chain: Vec<Block> = if Path::new(chain_path).exists() {
        let data = fs::read_to_string(chain_path).expect("無法讀取鏈檔案");
        serde_json::from_str(&data).expect("鏈格式錯誤")
    } else {
        Vec::new()
    };

    // ⛓️ 建立新區塊
    let previous_hash = chain.last().map(|b| b.data_hash.clone()).unwrap_or("0000000000000000".to_string());
    let index = chain.len() as u64;
    let block = create_block(index, previous_hash, hash.clone(), input_path.clone());

    // ⛓️ 加入鏈並儲存
    chain.push(block.clone());
    let chain_json = serde_json::to_string_pretty(&chain).expect("鏈序列化失敗");
    fs::write(chain_path, chain_json).expect("無法寫入鏈檔案");

    // 📦 儲存單篇 metadata
    let output_path = format!(
        "output/{}.json",
        Path::new(&input_path).file_stem().unwrap().to_str().unwrap()
    );
    let json = serde_json::to_string_pretty(&block.metadata).expect("JSON 序列化失敗");
    fs::write(&output_path, json).expect("無法寫入 JSON 檔案");

    println!("⛓️ 區塊已加入鏈，總區塊數：{}", chain.len());
    println!("📦 Metadata 儲存至：{}", output_path);
}

/// 驗證區塊鏈是否正確串接
fn verify_chain(path: &str) {
    println!("🔍 驗證中...");

    if !Path::new(path).exists() {
        eprintln!("❌ 找不到鏈檔案：{}", path);
        std::process::exit(1);
    }

    let data = fs::read_to_string(path).expect("無法讀取鏈檔案");
    let chain: Vec<Block> = serde_json::from_str(&data).expect("鏈格式錯誤");

    for i in 1..chain.len() {
        let prev = &chain[i - 1];
        let curr = &chain[i];

        if curr.previous_hash != prev.data_hash {
            eprintln!("❌ 區塊 {} 的 previous_hash 不正確", curr.index);
            std::process::exit(1);
        } else {
            println!("✅ 區塊 {} 正確連接", curr.index);
        }
    }

    println!("🎉 區塊鏈驗證成功，共 {} 個區塊", chain.len());
}

fn check_diary(filename: &str, chain_path: &str) {
    println!("🔍 檢查日記是否已上鏈：{}", filename);

    if !Path::new(chain_path).exists() {
        eprintln!("❌ 找不到鏈檔案：{}", chain_path);
        std::process::exit(1);
    }

    let data = fs::read_to_string(chain_path).expect("無法讀取鏈檔案");
    let chain: Vec<Block> = serde_json::from_str(&data).expect("鏈格式錯誤");

    for block in &chain {
        if block.metadata.filename == filename {
            println!(
                "✅ 已上鏈於區塊 #{}，hash: {}",
                block.index, block.data_hash
            );
            return;
        }
    }

    println!("❌ 尚未上鏈");
}

fn batch_process(diary_dir: &str, chain_path: &str) {
    println!("📂 批次處理資料夾：{}", diary_dir);

    let mut chain: Vec<Block> = if Path::new(chain_path).exists() {
        let data = fs::read_to_string(chain_path).expect("無法讀取鏈檔案");
        serde_json::from_str(&data).expect("鏈格式錯誤")
    } else {
        Vec::new()
    };



    let paths = fs::read_dir(diary_dir).expect("無法讀取 diary 資料夾");

    let key = b"anexampleverysecurekey12345678!!"; // 32 bytes
    let mut count = 0;

    for entry in paths {
    let path = entry.unwrap().path();
    if path.extension() != Some(OsStr::new("md")) {
        continue;
    }

    let filename = path.file_name().unwrap().to_str().unwrap();
    let stem = path.file_stem().unwrap().to_str().unwrap();
    let output_path = format!("output/{}.json", stem);

    if Path::new(&output_path).exists() {
        println!("⏭️ 已上鏈，略過：{}", filename);
        continue;
    }

    let content = fs::read_to_string(&path).expect("無法讀取日記檔案");
    let (_ciphertext, hash) = encrypt_and_hash(&content, key);

    let prev_hash = chain.last()
        .map(|b| b.data_hash.clone())
        .unwrap_or("0000000000000000".to_string());

    let block = create_block(chain.len() as u64, prev_hash, hash.clone(), filename.to_string());
    chain.push(block.clone());

let chain_json = serde_json::to_string_pretty(&chain).expect("鏈序列化失敗");
fs::write(chain_path, chain_json).expect("無法寫入鏈檔案");

    println!("✅ 上鏈成功：{}", filename);
    count += 1;
	
	println!("🎉 批次處理完成，共新增 {} 筆日記上鏈", count);
}
}

fn export_chain_to_csv(chain_path: &str, csv_path: &str) {
    println!("📤 匯出區塊鏈至 CSV：{}", csv_path);

    if !Path::new(chain_path).exists() {
        eprintln!("❌ 找不到鏈檔案：{}", chain_path);
        std::process::exit(1);
    }

    let data = fs::read_to_string(chain_path).expect("無法讀取鏈檔案");
    let chain: Vec<Block> = serde_json::from_str(&data).expect("鏈格式錯誤");

    let mut csv = String::from("index,filename,timestamp,hash,previous_hash\n");

    for block in &chain {
        let line = format!(
            "{},{},{},{},{}\n",
            block.index,
            block.metadata.filename,
            block.metadata.timestamp,
            block.data_hash,
            block.previous_hash
        );
        csv.push_str(&line);
    }

    fs::write(csv_path, csv).expect("無法寫入 CSV 檔案");
    println!("✅ 已匯出 {} 筆區塊資料至 {}", chain.len(), csv_path);
}

fn lock_chain_file(path: &str) {
    println!("🔒 嘗試鎖定鏈檔案：{}", path);

    if !Path::new(path).exists() {
        eprintln!("❌ 找不到鏈檔案：{}", path);
        std::process::exit(1);
    }

    let metadata = fs::metadata(path).expect("無法讀取檔案 metadata");
    let mut permissions = metadata.permissions();
    permissions.set_readonly(true);

    fs::set_permissions(path, permissions).expect("無法設定唯讀權限");

    println!("✅ 已將 {} 設為唯讀", path);
}
