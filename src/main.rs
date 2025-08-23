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
        Protocol::RaydiumAmmV4,
    ];

    println!("Protocols to monitor: {:?}", protocols);

    // Filter accounts
    let account_include = vec![
        PUMPFUN_PROGRAM_ID.to_string(),        // Listen to pumpfun program ID
        PUMPSWAP_PROGRAM_ID.to_string(),       // Listen to pumpswap program ID
        BONK_PROGRAM_ID.to_string(),           // Listen to bonk program ID
        RAYDIUM_CPMM_PROGRAM_ID.to_string(),   // Listen to raydium_cpmm program ID
        RAYDIUM_CLMM_PROGRAM_ID.to_string(),   // Listen to raydium_clmm program ID
        RAYDIUM_AMM_V4_PROGRAM_ID.to_string(), // Listen to raydium_amm_v4 program ID
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

    // Event filtering
    // No event filtering, includes all events
    let event_type_filter = None;
    // Only include PumpSwapBuy events and PumpSwapSell events
    // let event_type_filter = EventTypeFilter { include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] };

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

    // 支持 stop 方法，测试代码 -  异步1000秒之后停止
    let grpc_clone = grpc.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        grpc_clone.stop().await;
    });

    println!("Waiting for Ctrl+C to stop...");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

async fn test_shreds() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to ShredStream events...");

    // Create low-latency configuration
    let mut config = StreamClientConfig::low_latency();
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

    // Event filtering
    // No event filtering, includes all events
    let event_type_filter = None;
    // Only include PumpSwapBuy events and PumpSwapSell events
    // let event_type_filter =
    //     EventTypeFilter { include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] };

    println!("Listening for events, press Ctrl+C to stop...");
    shred_stream.shredstream_subscribe::<_, fn(Vec<Box<dyn UnifiedEvent>>)>(protocols, None,event_type_filter, callback,None).await?;

    // 支持 stop 方法，测试代码 - 异步1000秒之后停止
    let shred_clone = shred_stream.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        shred_clone.stop().await;
    });

    println!("Waiting for Ctrl+C to stop...");
    tokio::signal::ctrl_c().await?;

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
