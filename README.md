# Solana Streamer
[中文](https://github.com/0xfnzero/solana-streamer/blob/main/README_CN.md) | [English](https://github.com/0xfnzero/solana-streamer/blob/main/README.md) | [Telegram](https://t.me/fnzero_group)

A lightweight Rust library for real-time event streaming from Solana DEX trading programs. This library provides efficient event parsing and subscription capabilities for PumpFun, PumpSwap, Bonk, and Raydium CPMM protocols.

## Project Features

1. **Real-time Event Streaming**: Subscribe to live trading events from multiple Solana DEX protocols
2. **Yellowstone gRPC Support**: High-performance event subscription using Yellowstone gRPC
3. **ShredStream Support**: Alternative event streaming using ShredStream protocol
4. **Multi-Protocol Support**: 
   - **PumpFun**: Meme coin trading platform events
   - **PumpSwap**: PumpFun's swap protocol events
   - **Bonk**: Token launch platform events (letsbonk.fun)
   - **Raydium CPMM**: Raydium's Concentrated Pool Market Maker events
   - **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker events
   - **Raydium AMM V4**: Raydium's Automated Market Maker V4 events
5. **Unified Event Interface**: Consistent event handling across all supported protocols
6. **Event Parsing System**: Automatic parsing and categorization of protocol-specific events
7. **High Performance**: Optimized for low-latency event processing
8. **Batch Processing Optimization**: Batch processing events to reduce callback overhead
9. **Performance Monitoring**: Built-in performance metrics monitoring, including event processing speed, etc.
10. **Memory Optimization**: Object pooling and caching mechanisms to reduce memory allocations
11. **Flexible Configuration System**: Support for custom batch sizes, backpressure strategies, channel sizes, and other parameters
12. **Preset Configurations**: Provides high-performance, low-latency, ordered processing, and other preset configurations
13. **Backpressure Handling**: Supports blocking, dropping, retrying, ordered, and other backpressure strategies
14. **Runtime Configuration Updates**: Supports dynamic configuration parameter updates at runtime
15. **Full Function Performance Monitoring**: All subscribe_events functions support performance monitoring, automatically collecting and reporting performance metrics

## Installation

### Direct Clone

Clone this project to your project directory:

```bash
cd your_project_root_directory
git clone https://github.com/0xfnzero/solana-streamer
```

Add the dependency to your `Cargo.toml`:

```toml
# Add to your Cargo.toml
solana-streamer-sdk = { path = "./solana-streamer", version = "0.2.3" }
```

### Use crates.io

```toml
# Add to your Cargo.toml
solana-streamer-sdk = "0.2.3"
```

## Usage Examples

### Quick Start - Parse Transaction Events

You can quickly test the library by running the built-in example that parses transaction events:

```bash
cargo run --example parse_tx_events
```

This example demonstrates:
- How to parse transaction data from Solana mainnet using RPC
- Event parsing for multiple protocols (PumpFun, PumpSwap, Bonk, Raydium CPMM/CLMM/AMM V4)
- Transaction details extraction including fees, logs, and compute units

The example uses a predefined transaction signature and shows how to extract protocol-specific events from the transaction data.

### Advanced Usage with Batch Processing and Backpressure

```rust
use solana_streamer_sdk::{
    match_event,
    streaming::{
        event_parser::{
            protocols::{
                bonk::{
                    parser::BONK_PROGRAM_ID, BonkMigrateToAmmEvent, BonkMigrateToCpswapEvent,
                    BonkPoolCreateEvent, BonkTradeEvent,
                },
                pumpfun::{
                    parser::PUMPFUN_PROGRAM_ID, PumpFunCreateTokenEvent, PumpFunMigrateEvent,
                    PumpFunTradeEvent,
                },
                pumpswap::{
                    parser::PUMPSWAP_PROGRAM_ID, PumpSwapBuyEvent, PumpSwapCreatePoolEvent,
                    PumpSwapDepositEvent, PumpSwapSellEvent, PumpSwapWithdrawEvent,
                },
                raydium_amm_v4::{
                    RaydiumAmmV4DepositEvent, RaydiumAmmV4Initialize2Event, RaydiumAmmV4SwapEvent,
                    RaydiumAmmV4WithdrawEvent, RaydiumAmmV4WithdrawPnlEvent,
                },
                raydium_clmm::{
                    parser::RAYDIUM_CLMM_PROGRAM_ID, RaydiumClmmClosePositionEvent,
                    RaydiumClmmCreatePoolEvent, RaydiumClmmDecreaseLiquidityV2Event,
                    RaydiumClmmIncreaseLiquidityV2Event, RaydiumClmmOpenPositionV2Event,
                    RaydiumClmmOpenPositionWithToken22NftEvent, RaydiumClmmSwapEvent,
                    RaydiumClmmSwapV2Event,
                },
                raydium_cpmm::{
                    parser::RAYDIUM_CPMM_PROGRAM_ID, RaydiumCpmmDepositEvent,
                    RaydiumCpmmInitializeEvent, RaydiumCpmmSwapEvent, RaydiumCpmmWithdrawEvent,
                },
                BlockMetaEvent,
            },
            Protocol, UnifiedEvent,
        },
        grpc::ClientConfig,
        shred_stream::ShredClientConfig,
        ShredStreamGrpc, YellowstoneGrpc,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Solana Streamer...");
    
    // Test Yellowstone gRPC with performance monitoring
    test_grpc().await?;
    
    // Test ShredStream with performance monitoring  
    test_shreds().await?;
    
    Ok(())
}

async fn test_grpc() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to Yellowstone gRPC events...");

    // Create low-latency configuration
    let mut config = ClientConfig::low_latency();
    // Enable performance monitoring, has performance overhead, disabled by default
    config.enable_metrics = true;
    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    println!("GRPC client created successfully");

    let callback = create_event_callback();

    // Will try to parse corresponding protocol events from transactions
    let protocols = vec![
        Protocol::PumpFun,
        Protocol::PumpSwap,
        Protocol::Bonk,
        Protocol::RaydiumCpmm,
        Protocol::RaydiumClmm,
    ];

    println!("Protocols to monitor: {:?}", protocols);

    // Filter accounts
    let account_include = vec![
        PUMPFUN_PROGRAM_ID.to_string(),      // Listen to pumpfun program ID
        PUMPSWAP_PROGRAM_ID.to_string(),     // Listen to pumpswap program ID
        BONK_PROGRAM_ID.to_string(),         // Listen to bonk program ID
        RAYDIUM_CPMM_PROGRAM_ID.to_string(), // Listen to raydium_cpmm program ID
        RAYDIUM_CLMM_PROGRAM_ID.to_string(), // Listen to raydium_clmm program ID
    ];
    let account_exclude = vec![];
    let account_required = vec![];

    println!("Starting to listen for events, press Ctrl+C to stop...");
    println!("Monitoring programs: {:?}", account_include);

    println!("Starting subscription...");

    grpc.subscribe_events_immediate(
        protocols,
        None,
        account_include,
        account_exclude,
        account_required,
        None,
        callback,
    )
    .await?;

    Ok(())
}

async fn test_shreds() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to ShredStream events...");

    // Create low-latency configuration
    let mut config = ShredClientConfig::low_latency();
    // Enable performance monitoring, has performance overhead, disabled by default
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

    println!("Listening for events, press Ctrl+C to stop...");
    shred_stream.shredstream_subscribe(protocols, None, callback).await?;

    Ok(())
}

fn create_event_callback() -> impl Fn(Box<dyn UnifiedEvent>) {
    |event: Box<dyn UnifiedEvent>| {
        println!("🎉 Event received! Type: {:?}, ID: {}", event.event_type(), event.id());
        match_event!(event, {
            // block meta
            BlockMetaEvent => |e: BlockMetaEvent| {
                println!("BlockMetaEvent: {e:?}");
            },
            // bonk
            BonkPoolCreateEvent => |e: BonkPoolCreateEvent| {
                // When using grpc, you can get block_time from each event
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
            // pumpfun
            PumpFunTradeEvent => |e: PumpFunTradeEvent| {
                println!("PumpFunTradeEvent: {e:?}");
            },
            PumpFunMigrateEvent => |e: PumpFunMigrateEvent| {
                println!("PumpFunMigrateEvent: {e:?}");
            },
            PumpFunCreateTokenEvent => |e: PumpFunCreateTokenEvent| {
                println!("PumpFunCreateTokenEvent: {e:?}");
            },
            // pumpswap
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
            // raydium_cpmm
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
            // raydium_clmm
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
            // raydium_amm_v4
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
        });
    }
}
```

## Supported Protocols

- **PumpFun**: Primary meme coin trading platform
- **PumpSwap**: PumpFun's swap protocol
- **Bonk**: Token launch platform (letsbonk.fun)
- **Raydium CPMM**: Raydium's Concentrated Pool Market Maker protocol
- **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker protocol
- **Raydium AMM V4**: Raydium's Automated Market Maker V4 protocol

## Event Streaming Services

- **Yellowstone gRPC**: High-performance Solana event streaming
- **ShredStream**: Alternative event streaming protocol

## Architecture Features

### Unified Event Interface

- **UnifiedEvent Trait**: All protocol events implement a common interface
- **Protocol Enum**: Easy identification of event sources
- **Event Factory**: Automatic event parsing and categorization

### Event Parsing System

- **Protocol-specific Parsers**: Dedicated parsers for each supported protocol
- **Event Factory**: Centralized event creation and parsing
- **Extensible Design**: Easy to add new protocols and event types

### Streaming Infrastructure

- **Yellowstone gRPC Client**: Optimized for Solana event streaming
- **ShredStream Client**: Alternative streaming implementation
- **Async Processing**: Non-blocking event handling

## Project Structure

```
src/
├── common/           # Common functionality and types
├── protos/           # Protocol buffer definitions
├── streaming/        # Event streaming system
│   ├── event_parser/ # Event parsing system
│   │   ├── common/   # Common event parsing tools
│   │   ├── core/     # Core parsing traits and interfaces
│   │   ├── protocols/# Protocol-specific parsers
│   │   │   ├── bonk/ # Bonk event parsing
│   │   │   ├── pumpfun/ # PumpFun event parsing
│   │   │   ├── pumpswap/ # PumpSwap event parsing
│   │   │   ├── raydium_amm_v4/ # Raydium AMM V4 event parsing
│   │   │   ├── raydium_cpmm/ # Raydium CPMM event parsing
│   │   │   └── raydium_clmm/ # Raydium CLMM event parsing
│   │   └── factory.rs # Parser factory
│   ├── shred_stream.rs # ShredStream client
│   ├── yellowstone_grpc.rs # Yellowstone gRPC client
│   └── yellowstone_sub_system.rs # Yellowstone subsystem
├── lib.rs            # Main library file
└── main.rs           # Example program
```

## License

MIT License

## Contact

- Project Repository: https://github.com/0xfnzero/solana-streamer
- Telegram Group: https://t.me/fnzero_group

## Performance Considerations

1. **Connection Management**: Properly handle connection lifecycle and reconnection
2. **Event Filtering**: Use protocol filtering to reduce unnecessary event processing
3. **Memory Management**: Implement appropriate cleanup for long-running streams
4. **Error Handling**: Robust error handling for network issues and service interruptions
5. **Batch Processing Optimization**: Use batch processing to reduce callback overhead and improve throughput
6. **Performance Monitoring**: Enable performance monitoring to identify bottlenecks and optimization opportunities

## Important Notes

1. **Network Stability**: Ensure stable network connection for continuous event streaming
2. **Rate Limiting**: Be aware of rate limits on public gRPC endpoints
3. **Error Recovery**: Implement proper error handling and reconnection logic
5. **Compliance**: Ensure compliance with relevant laws and regulations

## Language Versions

- [English](README.md)
- [中文](README_CN.md)
