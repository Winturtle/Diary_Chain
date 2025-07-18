# 📝 DiaryChain - Rust CLI 區塊鏈日記工具

DiaryChain 是一款以 Rust 編寫的命令列工具，專為加密日記、不可竄改紀錄與區塊鏈式驗證而設計。支援批次處理、鏈驗證、CSV 匯出與唯讀保護等功能。

---

## 🚀 功能特色

- 🔐 加密 Markdown 日記並產生 hash
- ⛓️ 串接區塊鏈並儲存 metadata
- 🧪 驗證鏈的正確性與連接性
- 📂 批次處理整個資料夾
- 📋 檢查某篇日記是否已上鏈
- 📤 匯出鏈為 CSV 報告
- 🔒 將 `chain.json` 設為唯讀防止竄改

---

## 🧰 安裝方式

```bash
git clone https://github.com/yourname/diary_chain.git
cd diary_chain
cargo build --release