use anyhow::Result;

pub struct SendCryptoRequest {
    pub token: String,
    pub amount: u64,
    pub to: String
}

pub trait CryptoWallet {

    async fn send(req: SendCryptoRequest)-> Result<()>;

}