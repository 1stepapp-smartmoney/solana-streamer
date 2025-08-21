# Solana Streamer
[中文](https://github.com/0xfnzero/solana-streamer/blob/main/README_CN.md) | [English](https://github.com/0xfnzero/solana-streamer/blob/main/README.md) | [Telegram](https://t.me/fnzero_group)

一个轻量级的 Rust 库，用于从 Solana DEX 交易程序中实时流式传输事件。该库为 PumpFun、PumpSwap、Bonk 和 Raydium CPMM 协议提供高效的事件解析和订阅功能。

## 项目特性

1. **实时事件流**: 订阅多个 Solana DEX 协议的实时交易事件
2. **Yellowstone gRPC 支持**: 使用 Yellowstone gRPC 进行高性能事件订阅
3. **ShredStream 支持**: 使用 ShredStream 协议进行替代事件流传输
4. **多协议支持**: 
   - **PumpFun**: 迷因币交易平台事件
   - **PumpSwap**: PumpFun 的交换协议事件
   - **Bonk**: 代币发布平台事件 (letsbonk.fun)
   - **Raydium CPMM**: Raydium 集中池做市商事件
   - **Raydium CLMM**: Raydium 集中流动性做市商事件
   - **Raydium AMM V4**: Raydium 自动做市商 V4 事件
5. **统一事件接口**: 在所有支持的协议中保持一致的事件处理
6. **事件解析系统**: 自动解析和分类协议特定事件
7. **账户状态监控**: 实时监控协议账户状态和配置变更
8. **交易与账户事件过滤**: 分别过滤交易事件和账户状态变化
9. **高性能**: 针对低延迟事件处理进行优化
10. **批处理优化**: 批量处理事件以减少回调开销
11. **性能监控**: 内置性能指标监控，包括事件处理速度等
12. **内存优化**: 对象池和缓存机制减少内存分配
13. **灵活配置系统**: 支持自定义批处理大小、背压策略、通道大小等参数
14. **预设配置**: 提供高性能、低延迟、有序处理等预设配置
15. **背压处理**: 支持阻塞、丢弃、重试、有序等多种背压策略
16. **运行时配置更新**: 支持在运行时动态更新配置参数
17. **全函数性能监控**: 所有subscribe_events函数都支持性能监控，自动收集和报告性能指标

## 安装

### 直接克隆

将项目克隆到您的项目目录：

```bash
cd your_project_root_directory
git clone https://github.com/0xfnzero/solana-streamer
```

在您的 `Cargo.toml` 中添加依赖：

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = { path = "./solana-streamer", version = "0.3.5" }
```

### 使用 crates.io

```toml
# 添加到您的 Cargo.toml
solana-streamer-sdk = "0.3.5"
```

## 使用示例

### 快速开始 - 解析交易事件

您可以通过运行内置示例来快速测试库的交易事件解析功能：

```bash
cargo run --example parse_tx_events
```

该示例演示了：
- 如何使用 RPC 从 Solana 主网解析交易数据
- 多协议事件解析（PumpFun、PumpSwap、Bonk、Raydium CPMM/CLMM/AMM V4）
- 交易详情提取，包括费用、日志和计算单元

该示例使用预定义的交易签名，展示如何从交易数据中提取协议特定的事件。

### 高级用法 - 完整示例

```rust
use solana_streamer_sdk::{
    match_event,
    streaming::{
        event_parser::{
            common::{filter::EventTypeFilter, EventType},
            protocols::{
                bonk::{
                    parser::BONK_PROGRAM_ID, BonkGlobalConfigAccountEvent, BonkMigrateToAmmEvent,
                    BonkMigrateToCpswapEvent, BonkPlatformConfigAccountEvent, BonkPoolCreateEvent,
                    BonkPoolStateAccountEvent, BonkTradeEvent,
                },
                pumpfun::{
                    parser::PUMPFUN_PROGRAM_ID, PumpFunBondingCurveAccountEvent,
                    PumpFunCreateTokenEvent, PumpFunGlobalAccountEvent, PumpFunMigrateEvent,
                    PumpFunTradeEvent,
                },
                pumpswap::{
                    parser::PUMPSWAP_PROGRAM_ID, PumpSwapBuyEvent, PumpSwapCreatePoolEvent,
                    PumpSwapDepositEvent, PumpSwapGlobalConfigAccountEvent,
                    PumpSwapPoolAccountEvent, PumpSwapSellEvent, PumpSwapWithdrawEvent,
                },
                raydium_amm_v4::{
                    parser::RAYDIUM_AMM_V4_PROGRAM_ID, RaydiumAmmV4AmmInfoAccountEvent,
                    RaydiumAmmV4DepositEvent, RaydiumAmmV4Initialize2Event, RaydiumAmmV4SwapEvent,
                    RaydiumAmmV4WithdrawEvent, RaydiumAmmV4WithdrawPnlEvent,
                },
                raydium_clmm::{
                    parser::RAYDIUM_CLMM_PROGRAM_ID, RaydiumClmmAmmConfigAccountEvent,
                    RaydiumClmmClosePositionEvent, RaydiumClmmCreatePoolEvent,
                    RaydiumClmmDecreaseLiquidityV2Event, RaydiumClmmIncreaseLiquidityV2Event,
                    RaydiumClmmOpenPositionV2Event, RaydiumClmmOpenPositionWithToken22NftEvent,
                    RaydiumClmmPoolStateAccountEvent, RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event,
                    RaydiumClmmTickArrayStateAccountEvent,
                },
                raydium_cpmm::{
                    parser::RAYDIUM_CPMM_PROGRAM_ID, RaydiumCpmmAmmConfigAccountEvent,
                    RaydiumCpmmDepositEvent, RaydiumCpmmInitializeEvent,
                    RaydiumCpmmPoolStateAccountEvent, RaydiumCpmmSwapEvent,
                    RaydiumCpmmWithdrawEvent,
                },
                BlockMetaEvent,
            },
            Protocol, UnifiedEvent,
        },
        grpc::ClientConfig,
        shred::StreamClientConfig,
        yellowstone_grpc::{AccountFilter, TransactionFilter},
        ShredStreamGrpc, YellowstoneGrpc,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Solana Streamer...");
    test_grpc().await?;
    test_shreds().await?;
    Ok(())
}

async fn test_grpc() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to Yellowstone gRPC events...");

    // 创建低延迟配置
    let mut config = ClientConfig::low_latency();
    // 启用性能监控, 有性能损耗, 默认关闭
    config.enable_metrics = true;
    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    println!("GRPC client created successfully");

    let callback = create_event_callback();

    // 将会从交易中尝试解析对应的协议事件
    let protocols = vec![
        Protocol::PumpFun,
        Protocol::PumpSwap,
        Protocol::Bonk,
        Protocol::RaydiumCpmm,
        Protocol::RaydiumClmm,
        Protocol::RaydiumAmmV4,
    ];

    println!("Protocols to monitor: {:?}", protocols);

    // 过滤账号
    let account_include = vec![
        PUMPFUN_PROGRAM_ID.to_string(),        // 监听 pumpfun 程序ID
        PUMPSWAP_PROGRAM_ID.to_string(),       // 监听 pumpswap 程序ID
        BONK_PROGRAM_ID.to_string(),           // 监听 bonk 程序ID
        RAYDIUM_CPMM_PROGRAM_ID.to_string(),   // 监听 raydium_cpmm 程序ID
        RAYDIUM_CLMM_PROGRAM_ID.to_string(),   // 监听 raydium_clmm 程序ID
        RAYDIUM_AMM_V4_PROGRAM_ID.to_string(), // 监听 raydium_amm_v4 程序ID
    ];
    let account_exclude = vec![];
    let account_required = vec![];

    // 监听交易数据
    let transaction_filter = TransactionFilter {
        account_include: account_include.clone(),
        account_exclude,
        account_required,
    };

    // 监听属于owner程序的账号数据 -> 账号事件监听
    let account_filter = AccountFilter { account: vec![], owner: account_include.clone() };

    // 事件过滤 - 可选
    // 不进行事件过滤，包含所有事件
    let event_type_filter = None;
    // 只包含PumpSwapBuy事件、PumpSwapSell事件
    // let event_type_filter = Some(EventTypeFilter { include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] });

    println!("Starting to listen for events, press Ctrl+C to stop...");
    println!("Monitoring programs: {:?}", account_include);

    println!("Starting subscription...");

    grpc.subscribe_events_immediate(
        protocols,
        None,
        transaction_filter,
        account_filter,
        event_type_filter,
        None,
        callback,
    )
    .await?;

    Ok(())
}

async fn test_shreds() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to ShredStream events...");

    // 创建低延迟配置
    let mut config = StreamClientConfig::low_latency();
    // 启用性能监控, 有性能损耗, 默认关闭
    config.enable_metrics = true;
    let shred_stream =
        ShredStreamGrpc::new_with_config("http://127.0.0.1:10800".to_string(), config).await?;

    let callback = create_event_callback();
    let protocols = vec![
        Protocol::PumpFun,
        Protocol::PumpSwap,
        Protocol::Bonk,
        Protocol::RaydiumCpmm,
        Protocol::RaydiumClmm,
        Protocol::RaydiumAmmV4,
    ];

    // 事件过滤
    // 不进行事件过滤，包含所有事件
    let event_type_filter = None;
    // 只包含PumpSwapBuy事件、PumpSwapSell事件
    // let event_type_filter =
    //     EventTypeFilter { include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] };

    println!("Listening for events, press Ctrl+C to stop...");
    shred_stream.shredstream_subscribe(protocols, None, event_type_filter, callback).await?;

    Ok(())
}

fn create_event_callback() -> impl Fn(Box<dyn UnifiedEvent>) {
    |event: Box<dyn UnifiedEvent>| {
        println!("🎉 Event received! Type: {:?}, ID: {}", event.event_type(), event.id());
        match_event!(event, {
            // -------------------------- block meta -----------------------
            BlockMetaEvent => |e: BlockMetaEvent| {
                println!("BlockMetaEvent: {e:?}");
            },
            // -------------------------- bonk -----------------------
            BonkPoolCreateEvent => |e: BonkPoolCreateEvent| {
                // 使用grpc的时候，可以从每个事件中获取到block_time
                println!("block_time: {:?}, block_time_ms: {:?}", e.metadata.block_time, e.metadata.block_time_ms);
                println!("BonkPoolCreateEvent: {:?}", e.base_mint_param.symbol);
            },
            BonkTradeEvent => |e: BonkTradeEvent| {
                println!("BonkTradeEvent: {e:?}");
            },
            BonkMigrateToAmmEvent => |e: BonkMigrateToAmmEvent| {
                println!("BonkMigrateToAmmEvent: {e:?}");
            },
            BonkMigrateToCpswapEvent => |e: BonkMigrateToCpswapEvent| {
                println!("BonkMigrateToCpswapEvent: {e:?}");
            },
            // -------------------------- pumpfun -----------------------
            PumpFunTradeEvent => |e: PumpFunTradeEvent| {
                println!("PumpFunTradeEvent: {e:?}");
            },
            PumpFunMigrateEvent => |e: PumpFunMigrateEvent| {
                println!("PumpFunMigrateEvent: {e:?}");
            },
            PumpFunCreateTokenEvent => |e: PumpFunCreateTokenEvent| {
                println!("PumpFunCreateTokenEvent: {e:?}");
            },
            // -------------------------- pumpswap -----------------------
            PumpSwapBuyEvent => |e: PumpSwapBuyEvent| {
                println!("Buy event: {e:?}");
            },
            PumpSwapSellEvent => |e: PumpSwapSellEvent| {
                println!("Sell event: {e:?}");
            },
            PumpSwapCreatePoolEvent => |e: PumpSwapCreatePoolEvent| {
                println!("CreatePool event: {e:?}");
            },
            PumpSwapDepositEvent => |e: PumpSwapDepositEvent| {
                println!("Deposit event: {e:?}");
            },
            PumpSwapWithdrawEvent => |e: PumpSwapWithdrawEvent| {
                println!("Withdraw event: {e:?}");
            },
            // -------------------------- raydium_cpmm -----------------------
            RaydiumCpmmSwapEvent => |e: RaydiumCpmmSwapEvent| {
                println!("RaydiumCpmmSwapEvent: {e:?}");
            },
            RaydiumCpmmDepositEvent => |e: RaydiumCpmmDepositEvent| {
                println!("RaydiumCpmmDepositEvent: {e:?}");
            },
            RaydiumCpmmInitializeEvent => |e: RaydiumCpmmInitializeEvent| {
                println!("RaydiumCpmmInitializeEvent: {e:?}");
            },
            RaydiumCpmmWithdrawEvent => |e: RaydiumCpmmWithdrawEvent| {
                println!("RaydiumCpmmWithdrawEvent: {e:?}");
            },
            // -------------------------- raydium_clmm -----------------------
            RaydiumClmmSwapEvent => |e: RaydiumClmmSwapEvent| {
                println!("RaydiumClmmSwapEvent: {e:?}");
            },
            RaydiumClmmSwapV2Event => |e: RaydiumClmmSwapV2Event| {
                println!("RaydiumClmmSwapV2Event: {e:?}");
            },
            RaydiumClmmClosePositionEvent => |e: RaydiumClmmClosePositionEvent| {
                println!("RaydiumClmmClosePositionEvent: {e:?}");
            },
            RaydiumClmmDecreaseLiquidityV2Event => |e: RaydiumClmmDecreaseLiquidityV2Event| {
                println!("RaydiumClmmDecreaseLiquidityV2Event: {e:?}");
            },
            RaydiumClmmCreatePoolEvent => |e: RaydiumClmmCreatePoolEvent| {
                println!("RaydiumClmmCreatePoolEvent: {e:?}");
            },
            RaydiumClmmIncreaseLiquidityV2Event => |e: RaydiumClmmIncreaseLiquidityV2Event| {
                println!("RaydiumClmmIncreaseLiquidityV2Event: {e:?}");
            },
            RaydiumClmmOpenPositionWithToken22NftEvent => |e: RaydiumClmmOpenPositionWithToken22NftEvent| {
                println!("RaydiumClmmOpenPositionWithToken22NftEvent: {e:?}");
            },
            RaydiumClmmOpenPositionV2Event => |e: RaydiumClmmOpenPositionV2Event| {
                println!("RaydiumClmmOpenPositionV2Event: {e:?}");
            },
            // -------------------------- raydium_amm_v4 -----------------------
            RaydiumAmmV4SwapEvent => |e: RaydiumAmmV4SwapEvent| {
                println!("RaydiumAmmV4SwapEvent: {e:?}");
            },
            RaydiumAmmV4DepositEvent => |e: RaydiumAmmV4DepositEvent| {
                println!("RaydiumAmmV4DepositEvent: {e:?}");
            },
            RaydiumAmmV4Initialize2Event => |e: RaydiumAmmV4Initialize2Event| {
                println!("RaydiumAmmV4Initialize2Event: {e:?}");
            },
            RaydiumAmmV4WithdrawEvent => |e: RaydiumAmmV4WithdrawEvent| {
                println!("RaydiumAmmV4WithdrawEvent: {e:?}");
            },
            RaydiumAmmV4WithdrawPnlEvent => |e: RaydiumAmmV4WithdrawPnlEvent| {
                println!("RaydiumAmmV4WithdrawPnlEvent: {e:?}");
            },
            // -------------------------- account -----------------------
            BonkPoolStateAccountEvent => |e: BonkPoolStateAccountEvent| {
                println!("BonkPoolStateAccountEvent: {e:?}");
            },
            BonkGlobalConfigAccountEvent => |e: BonkGlobalConfigAccountEvent| {
                println!("BonkGlobalConfigAccountEvent: {e:?}");
            },
            BonkPlatformConfigAccountEvent => |e: BonkPlatformConfigAccountEvent| {
                println!("BonkPlatformConfigAccountEvent: {e:?}");
            },
            PumpSwapGlobalConfigAccountEvent => |e: PumpSwapGlobalConfigAccountEvent| {
                println!("PumpSwapGlobalConfigAccountEvent: {e:?}");
            },
            PumpSwapPoolAccountEvent => |e: PumpSwapPoolAccountEvent| {
                println!("PumpSwapPoolAccountEvent: {e:?}");
            },
            PumpFunBondingCurveAccountEvent => |e: PumpFunBondingCurveAccountEvent| {
                println!("PumpFunBondingCurveAccountEvent: {e:?}");
            },
            PumpFunGlobalAccountEvent => |e: PumpFunGlobalAccountEvent| {
                println!("PumpFunGlobalAccountEvent: {e:?}");
            },
            RaydiumAmmV4AmmInfoAccountEvent => |e: RaydiumAmmV4AmmInfoAccountEvent| {
                println!("RaydiumAmmV4AmmInfoAccountEvent: {e:?}");
            },
            RaydiumClmmAmmConfigAccountEvent => |e: RaydiumClmmAmmConfigAccountEvent| {
                println!("RaydiumClmmAmmConfigAccountEvent: {e:?}");
            },
            RaydiumClmmPoolStateAccountEvent => |e: RaydiumClmmPoolStateAccountEvent| {
                println!("RaydiumClmmPoolStateAccountEvent: {e:?}");
            },
            RaydiumClmmTickArrayStateAccountEvent => |e: RaydiumClmmTickArrayStateAccountEvent| {
                println!("RaydiumClmmTickArrayStateAccountEvent: {e:?}");
            },
            RaydiumCpmmAmmConfigAccountEvent => |e: RaydiumCpmmAmmConfigAccountEvent| {
                println!("RaydiumCpmmAmmConfigAccountEvent: {e:?}");
            },
            RaydiumCpmmPoolStateAccountEvent => |e: RaydiumCpmmPoolStateAccountEvent| {
                println!("RaydiumCpmmPoolStateAccountEvent: {e:?}");
            },
        });
    }
}
```

### 事件过滤

库支持灵活的事件过滤以减少处理开销并提升性能：

#### 基础过滤

```rust
use solana_streamer_sdk::streaming::event_parser::common::{filter::EventTypeFilter, EventType};

// 无过滤 - 接收所有事件
let event_type_filter = None;

// 过滤特定事件类型 - 只接收 PumpSwap 买入/卖出事件
let event_type_filter = Some(EventTypeFilter { 
    include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] 
});
```

#### 性能影响

事件过滤可以带来显著的性能提升：
- **减少 60-80%** 的不必要事件处理
- **降低内存使用** 通过过滤掉无关事件
- **减少网络带宽** 在分布式环境中
- **更好的专注性** 只处理对应用有意义的事件

#### 按使用场景的过滤示例

**交易机器人（专注交易事件）**
```rust
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapBuy,
        EventType::PumpSwapSell,
        EventType::PumpFunTrade,
        EventType::RaydiumCpmmSwap,
        EventType::RaydiumClmmSwap,
        EventType::RaydiumAmmV4Swap,
        .....
    ] 
});
```

**池监控（专注流动性事件）**
```rust
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapCreatePool,
        EventType::PumpSwapDeposit,
        EventType::PumpSwapWithdraw,
        EventType::RaydiumCpmmInitialize,
        EventType::RaydiumCpmmDeposit,
        EventType::RaydiumCpmmWithdraw,
        EventType::RaydiumClmmCreatePool,
        ......
    ] 
});
```

## 支持的协议

- **PumpFun**: 主要迷因币交易平台
- **PumpSwap**: PumpFun 的交换协议
- **Bonk**: 代币发布平台 (letsbonk.fun)
- **Raydium CPMM**: Raydium 集中池做市商协议
- **Raydium CLMM**: Raydium 集中流动性做市商协议
- **Raydium AMM V4**: Raydium 自动做市商 V4 协议

## 事件流服务

- **Yellowstone gRPC**: 高性能 Solana 事件流
- **ShredStream**: 替代事件流协议

## 架构特性

### 统一事件接口

- **UnifiedEvent Trait**: 所有协议事件实现通用接口
- **Protocol Enum**: 轻松识别事件来源
- **Event Factory**: 自动事件解析和分类

### 事件解析系统

- **协议特定解析器**: 每个支持协议的专用解析器
- **事件工厂**: 集中式事件创建和解析
- **可扩展设计**: 易于添加新协议和事件类型

### 流基础设施

- **Yellowstone gRPC 客户端**: 针对 Solana 事件流优化
- **ShredStream 客户端**: 替代流实现
- **异步处理**: 非阻塞事件处理

## 项目结构

```
src/
├── common/           # 通用功能和类型
├── protos/           # Protocol buffer 定义
├── streaming/        # 事件流系统
│   ├── event_parser/ # 事件解析系统
│   │   ├── common/   # 通用事件解析工具
│   │   ├── core/     # 核心解析特征和接口
│   │   ├── protocols/# 协议特定解析器
│   │   │   ├── bonk/ # Bonk 事件解析
│   │   │   ├── pumpfun/ # PumpFun 事件解析
│   │   │   ├── pumpswap/ # PumpSwap 事件解析
│   │   │   ├── raydium_amm_v4/ # Raydium AMM V4 事件解析
│   │   │   ├── raydium_cpmm/ # Raydium CPMM 事件解析
│   │   │   └── raydium_clmm/ # Raydium CLMM 事件解析
│   │   └── factory.rs # 解析器工厂
│   ├── shred_stream.rs # ShredStream 客户端
│   ├── yellowstone_grpc.rs # Yellowstone gRPC 客户端
│   └── yellowstone_sub_system.rs # Yellowstone 子系统
├── lib.rs            # 主库文件
└── main.rs           # 示例程序
```

## 性能考虑

1. **连接管理**: 正确处理连接生命周期和重连
2. **事件过滤**: 使用协议过滤减少不必要的事件处理
3. **内存管理**: 为长时间运行的流实现适当的清理
4. **错误处理**: 对网络问题和服务中断进行健壮的错误处理
5. **批处理优化**: 使用批处理减少回调开销，提高吞吐量
6. **性能监控**: 启用性能监控以识别瓶颈和优化机会

## 许可证

MIT 许可证

## 联系方式

- 项目仓库: https://github.com/0xfnzero/solana-streamer
- Telegram 群组: https://t.me/fnzero_group

## 重要注意事项

1. **网络稳定性**: 确保稳定的网络连接以进行连续的事件流传输
2. **速率限制**: 注意公共 gRPC 端点的速率限制
3. **错误恢复**: 实现适当的错误处理和重连逻辑
5. **合规性**: 确保遵守相关法律法规

## 语言版本

- [English](README.md)
- [中文](README_CN.md)