use crate::chains::aptos::AptosWallet;
use crate::chains::traits::CryptoWallet;

pub mod aptos;
pub mod traits;

pub enum  TumaSupportedChains {
    APTOS(AptosWallet)
}