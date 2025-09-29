use std::env;
use std::str::FromStr;
use std::time::Duration;
use anyhow::{Result, anyhow};
use aptos_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey};
use aptos_crypto::ValidCryptoMaterialStringExt;
use aptos_rust_sdk::client::builder::AptosClientBuilder;
use aptos_rust_sdk::client::config::AptosNetwork;
use aptos_rust_sdk::client::rest_api::AptosFullnodeClient;
use aptos_rust_sdk_types::api_types::account::AccountResource;
use aptos_rust_sdk_types::api_types::address::AccountAddress;
use aptos_rust_sdk_types::api_types::chain_id::ChainId;
use aptos_rust_sdk_types::api_types::module_id::ModuleId;
use aptos_rust_sdk_types::api_types::transaction::{EntryFunction, GenerateSigningMessage, RawTransaction, SignedTransaction, TransactionPayload};
use aptos_rust_sdk_types::api_types::transaction_authenticator::{AccountAuthenticator, AuthenticationKey, TransactionAuthenticator};
use aptos_rust_sdk_types::api_types::type_tag::TypeTag;
use serde_json::Value;

fn parse_fixed<S: AsRef<str>>(s: S, scale: Option<u64>) -> Result<u64, &'static str> {
    let scale = scale.unwrap_or(100_000_000);

    // quick sanity: scale must be a power of ten
    if scale == 0 || scale % 10 != 0 {
        return Err("bad scale");
    }
    let precision = scale.trailing_zeros() as usize; // 10⁸ -> 8

    let s = s.as_ref();
    let mut parts = s.split('.');
    let int_part  = parts.next().unwrap_or("0");
    let frac_part = parts.next().unwrap_or("");
    if parts.next().is_some() { return Err("multiple dots"); }

    let mut units = int_part.parse::<u64>().map_err(|_| "bad int")?
        .checked_mul(scale).ok_or("overflow")?;

    // right-pad/truncate fractional digits to the required precision
    let mut frac_str = frac_part.to_string();
    if frac_str.len() > precision { frac_str.truncate(precision); }
    while frac_str.len() < precision { frac_str.push('0'); }

    let frac = frac_str.parse::<u64>().map_err(|_| "bad frac")?;
    units = units.checked_add(frac).ok_or("overflow")?;
    Ok(units)
}

pub struct SendTokenTransactionArgs {
    pub to_account: String,
    pub amount: String,
    pub token_type: Option<String>,
    pub scale: Option<u64>,
    pub on_ramp_request_id: String
}

pub struct SendFungibleTokenArgs {
    pub to_account: String,
    pub amount: String,
    pub token: String,
    pub scale: Option<u64>,
    pub on_ramp_request_id: String
}

pub enum WalletTransaction {
    SendToken(SendTokenTransactionArgs),
    SendFungibleToken(SendFungibleTokenArgs)
}

pub struct AptosWallet {
    pub client: AptosFullnodeClient,
    pub key: Ed25519PrivateKey,
    pub public_key: Ed25519PublicKey,
    pub auth_key: AuthenticationKey,
    pub sender: AccountAddress,
    pub chain_id: ChainId,
    pub tooma_module_id: ModuleId
}


impl AptosWallet {
    pub fn new()-> Result<Self> {

        let network_val = env::var("NETWORK").unwrap_or("testnet".to_string());
        let tooma_contract_address = env::var("TOOMA_CONTRACT_ADDRESS").expect("ISSUER CONTRACT ADDRESS NOT PROVIDED");
        let private_key = env::var("PRIVATE_KEY_DO_NOT_EXPOSE").expect("PRIVATE KEY NOT FOUND");
        let aptos_api_key = match env::var("APTOS_API_KEY") {
            Ok(k)=>Some(k),
            Err(_)=>None
        };
        let (network, chain_id) = if network_val.eq(&"testnet".to_string()) { (AptosNetwork::testnet(), ChainId::Testnet)} else {(AptosNetwork::mainnet(), ChainId::Mainnet)};

        let mut builder = AptosClientBuilder::new(network);
        if let Some(k) = aptos_api_key  {
            builder = builder.api_key(&k).unwrap_or_else(|_| {
                println!("Unable to set api key");
                panic!("failed to get builder")
            })
        }
        let client = builder.build();

        let ed25519_key = Ed25519PrivateKey::from_encoded_string(&private_key)?;

        let public_key = Ed25519PublicKey::from(&ed25519_key);

        let authentication_key = AuthenticationKey::ed25519(&public_key);

        let sender = authentication_key.account_address();

        let contract_address = AccountAddress::from_str(&tooma_contract_address)?;

        let module_id = ModuleId::new(contract_address, "issuer".to_string());

        Ok(Self {
            client,
            key: ed25519_key,
            public_key,
            auth_key: authentication_key,
            sender,
            chain_id,
            tooma_module_id: module_id
        })
    }

    pub async fn get_account_resources(&self) -> Result<Vec<AccountResource>> {

        let resource = self.client.get_account_resources(self.sender.to_string()).await?.into_inner();
        Ok(resource)
    }

    pub async fn get_sequence_number(&self, account_resources: &Vec<AccountResource>) -> Result<u64> {

        let sequence_number = account_resources.iter()
            .find(|r| r.type_ == "0x1::account::Account")
            .unwrap()
            .data
            .get("sequence_number")
            .unwrap()
            .as_str()
            .unwrap()
            .parse::<u64>()
            ?;

        Ok(sequence_number)
    }

    async fn get_transaction_status(&self, hash: String, repeat: Option<u64>) -> Result<bool> {
        let mut count = repeat.unwrap_or(0);
        loop {
            if count >= 5 {
                return Ok(false);
            }
            let full_node_response = self.client.get_transaction_by_hash(hash.clone()).await?.into_inner();
            if let Value::Object(response) = full_node_response {

                if let Some(success_value) = response.get("success") {
                    if let &Value::Bool(success) = success_value {
                        return Ok(success);
                    }
                } else if let Some(tx_type) = response.get("type") {
                    if let Value::String(tx_type_value) = tx_type {
                        if tx_type_value.eq("pending_transaction") {
                            tokio::time::sleep(Duration::from_secs(1)).await.await;
                            count += 1;
                            continue;
                        }
                    }
                }
            }
            return Ok(false);
        }
    }

    pub async fn send(&mut self, transaction_payload: WalletTransaction)->Result<String>{
        let state = self.client.get_state().await?;
        let account_resources =  self.get_account_resources().await?;
        let sequence_number = self.get_sequence_number(&account_resources).await?;
        let max_gas_amount = 50000;
        let gas_unit_price = 100;
        let expiration_timestamp_secs = state.timestamp_usecs / 1000 / 1000 + 60 * 10;

        let mut payload: TransactionPayload;
        match transaction_payload {
            WalletTransaction::SendToken(args)=>{
                let parsed_amount = parse_fixed(args.amount, args.scale).unwrap();

                let to_address = AccountAddress::from_str(&args.to_account)?;
                let mut type_args: Vec<TypeTag> = vec![];
                if let Some(ref token_type) = args.token_type {
                    let type_arg = TypeTag::from_str(&token_type)?;
                    type_args.push(type_arg)
                }

                let tuma_id = self.tooma_module_id.clone();
                payload = TransactionPayload::EntryFunction(
                    EntryFunction::new(
                        tuma_id,
                        "transfer_coins".to_string(),
                        type_args,
                        vec![
                            to_address.to_vec(),
                            Vec::from(parsed_amount.to_le_bytes()),
                            args.on_ramp_request_id.into_bytes()
                        ]
                    )
                )

            },
            WalletTransaction::SendFungibleToken(args)=> {
                let parsed_amount = parse_fixed(args.amount, args.scale).unwrap();

                let to_address = AccountAddress::from_str(&args.to_account)?;

                let metadata = AccountAddress::from_str(&args.token)?;
                let tuma_module_id = self.tooma_module_id.clone();
                payload = TransactionPayload::EntryFunction(
                    EntryFunction::new(
                        tuma_module_id,
                        "transfer_fungible".to_string(),
                        vec![
                            TypeTag::from_str("0x1::fungible_asset::Metadata")?
                        ],
                        vec![
                            metadata.to_vec(),
                            to_address.to_vec(),
                            Vec::from(parsed_amount.to_le_bytes()),
                            args.on_ramp_request_id.into_bytes()
                        ]
                    )
                )
            }
        }

        let raw_txn = RawTransaction::new(
            self.sender,
            sequence_number,
            payload,
            max_gas_amount,
            gas_unit_price,
            expiration_timestamp_secs,
            self.chain_id
        );

        let message = raw_txn.generate_signing_message()?;

        let signature = self.key.sign_message(&message);

        let public_key = Ed25519PublicKey::from(&self.key);
        let simulate_transaction = self.client.simulate_transaction(
            SignedTransaction::new(
                raw_txn.clone(),
                TransactionAuthenticator::single_sender(AccountAuthenticator::no_authenticator())
            )
        ).await?;

        println!("Simulate Transaction {:?}", simulate_transaction);

        let transaction = self.client.submit_transaction(
            SignedTransaction::new(
                raw_txn.clone(),
                TransactionAuthenticator::ed25519(public_key.clone(), signature)
            )
        ).await?;



        if let Value::Object(data) = &transaction.inner() {

            if let Value::String(hash) = data.get("hash").unwrap() {

                let success = self.get_transaction_status(hash.clone(), None).await?;

                if success {
                    return Ok(hash.clone())
                }

                return Err(anyhow!("transaction failed"))
            };

        };

        // self.client.get_transaction_by_hash()


        Err(anyhow!("Unable to retrieve transaction details"))
    }
}