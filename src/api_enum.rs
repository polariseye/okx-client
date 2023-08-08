use crate::utils::request_limit::*;

macro_rules! api_define {
    (
        $enum_name:ident
        $(
            (
                $(#[$docs:meta])*
                $name:ident $(,$init:expr)+
            );
        )+
    ) => {
        pub enum $enum_name {
            $($name,)+
        }

        impl $enum_name {
            #[allow(unused)]
            pub fn create_limit_mgr() -> LimitMgr {
                let mut mgr = LimitMgr::new();
                $(
                    mgr.add_limit($enum_name::$name as u32, vec![$($init),+]);
                )+;

                mgr
            }
        }
    }
}

api_define!{
    AccountApi
    (AccountBalance, WeightLimit::new(10, 2));
    (AccountPositions, WeightLimit::new(10, 2));
    (AccountSetLeverage, WeightLimit::new(20, 2));
    (AccountPositionsHistory, WeightLimit::new(1, 10));
    (TradeOrdersPending, WeightLimit::new(60, 2));
    (TradeOrdersHistory, WeightLimit::new(40, 2));
    // 限速规则（期权以外）：UserID + Instrument ID
    // 限速规则（只限期权）：UserID + Instrument Family
    (TradeCancelBatchOrders, WeightLimit::new(300, 2));
    // 限速规则（期权以外）：UserID + Instrument ID
    // 限速规则（只限期权）：UserID + Instrument Family
    (TradePlaceOrder, WeightLimit::new(60, 2));
    // 跟单交易带单合约的限速：1个/2s
    // 限速规则（期权以外）：UserID + Instrument ID
    // 限速规则（只限期权）：UserID + Instrument Family
    (TradePlaceBatchOrders, WeightLimit::new(300, 2));
    // 限速规则（期权以外）：UserID + Instrument ID
    // 限速规则（只限期权）：UserID + Instrument Family
    (TradeGetOrder, WeightLimit::new(60, 2));
    // 限速规则：UserID + Instrument ID
    (TradeAmendOrder, WeightLimit::new(60, 2));
}

api_define!{
    PublicApi
    (MarketTickers, WeightLimit::new(20, 2));
    (MarketTicker, WeightLimit::new(20, 2));
    (MarketTrades, WeightLimit::new(40, 2));
    // 限速规则：IP +instType
    (PublicInstruments, WeightLimit::new(20, 2));
}