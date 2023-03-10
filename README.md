# ustc-adbs-lab-rust

下载源代码和测试数据：
```
git clone https://github.com/ZonePG/ustc-adbs-lab-rust.git
cd ustc-adbs-lab-rust
```

代码目录如下：
```
├── Cargo.toml // Cargo 配置文件
├── README.md
├── data       // 测试数据文件
├── data-5w-1000-zipf.txt
├── data-5w-1w-zipf.txt
└── data-5w-50w-zipf.txt
src
├── buffer_manager.rs       // 缓存管理器
├── config.rs               // 命令行参数配置
├── data_storage_manager.rs // 存储管理器
├── main.rs
├── page.rs                 // page 结构体
└── replacer
    ├── clock_replacer.rs   // Clock 置换算法
    ├── lru_replacer.rs     // LRU 置换算法
    ├── mod.rs
    └── replacer.rs         // 置换算法接口
```

## Run

编译运行格式如下，其中 `lru` 和 `clock` 分别表示 LRU 和 Clock 置换算法，`file_path` 表示测试数据文件路径。
```
cargo run --release -- [lru|clock] [file_path]
```

例如：
```
cargo run --release -- clock data/data-5w-50w-zipf.txt
```

## Test

运行所有组件测试用例：
```
cargo test
```

运行指定测试用例，并允许标准输出：
```
cargo test  -- --nocapture test_func_name
cargo test  -- --nocapture buffer_manager
```
