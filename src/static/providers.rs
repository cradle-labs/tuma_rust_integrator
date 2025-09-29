use crate::payment_provider::provider::{FiatPaymentProvider, PaymentProviderType};
use crate::r#static::currency::CurrencyStaticData;

pub struct StaticProviderData {
    pub providers: Vec<FiatPaymentProvider>
}

impl StaticProviderData {
    pub fn new()-> Self{
        let currency_static_data = CurrencyStaticData::new();

        Self {
            providers: vec![
                FiatPaymentProvider {
                    supported_currency: currency_static_data.get_currency_by_id("kes".to_string()).unwrap(),
                    description: "Safaricom".to_string(),
                    id: "safaricom".to_string(),
                    name: "Safaricom".to_string(),
                    provider_type: PaymentProviderType::MobileMoney
                }
            ]
        }
    }


    pub fn get_id(&self, id: &str)->Option<FiatPaymentProvider> {
        match self.providers.iter().find(|c|c.id == id.to_string()) {
            Some(p)=>Some(p.clone()),
            None=>None
        }
    }

}