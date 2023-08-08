pub enum APiEnum {
    AccountBalance,
    AccountPositions,
    AccountSetLeverage,
    AccountPositionsHistory,
    AccountConfig,
    TradeOrdersPending,
    TradeOrdersHistory,
    // 限速：300个/2s
    // 限速规则（期权以外）：UserID + Instrument ID
    // 限速规则（只限期权）：UserID + Instrument Family
    TradeCancelBatchOrders,
    // 限速规则（期权以外）：UserID + Instrument ID
    // 限速规则（只限期权）：UserID + Instrument Family
    TradePlaceOrder,
    // 限速：300个/2s
    // 跟单交易带单合约的限速：1个/2s
    // 限速规则（期权以外）：UserID + Instrument ID
    // 限速规则（只限期权）：UserID + Instrument Family
    TradePlaceBatchOrders,
    // 限速规则（期权以外）：UserID + Instrument ID
    // 限速规则（只限期权）：UserID + Instrument Family
    TradeGetOrder,
    // 限速规则：UserID + Instrument ID
    TradeAmendOrder,
    MarketTickers,
    MarketTicker,
    MarketTrades,
    MarketBooks,
    PublicInstruments,
}