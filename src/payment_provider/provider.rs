use serde::{Deserialize, Serialize};
use crate::controller::currency_controller::Currency;

#[derive(Deserialize,Serialize,Clone,Debug)]
pub enum PaymentProviderType {
    Bank,
    MobileMoney
}

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct FiatPaymentProvider {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider_type: PaymentProviderType,
    pub supported_currency: Currency
}