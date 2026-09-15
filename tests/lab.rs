#![allow(deprecated)] 

use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::signer::Signer;
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction::create_account;
use spl_token::instruction::{initialize_mint, mint_to, transfer_checked};
use spl_token::state::Mint;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use mpl_core::instructions::{CreateV1Builder, UpdateV1Builder, TransferV1Builder, BurnV1Builder};
use mpl_core::accounts::BaseAssetV1;

const URI: &str = "https://raw.githubusercontent.com/shaurya35/spl-nft-lab/main/assets/metadata.json";

fn client() -> RpcClient {
    RpcClient::new("https://api.devnet.solana.com".to_string())
}

fn payer() -> Keypair {
    read_keypair_file("devnet-wallet.json").expect("wallet not found at crate root")
}

fn mint_asset(client: &RpcClient, payer: &Keypair, name: &str) -> Keypair {
    let asset = Keypair::new();
    let ix = CreateV1Builder::new()
        .asset(asset.pubkey())
        .payer(payer.pubkey())
        .owner(Some(payer.pubkey()))
        .name(name.to_string())
        .uri(URI.to_string())
        .instruction();
    let bh = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[payer, &asset], bh);
    client.send_and_confirm_transaction(&tx).unwrap();
    asset
}

#[test]
fn mint_and_transfer_spl() {
    let client = client();
    let payer = payer();

    let mint = Keypair::new();
    let rent = client.get_minimum_balance_for_rent_exemption(Mint::LEN).unwrap();
    let create_mint_ix = create_account(
        &payer.pubkey(), 
        &mint.pubkey(), 
        rent, 
        Mint::LEN as u64, 
        &spl_token::id()
    );

    let init_mint_ix = initialize_mint(
        &spl_token::id(),
        &mint.pubkey(),
        &payer.pubkey(),
        Some(&payer.pubkey()),
        6
    ).unwrap();

    let payer_ata = get_associated_token_address(
        &payer.pubkey(), 
        &mint.pubkey()
    );

    let create_payer_ata_ix = create_associated_token_account(
        &payer.pubkey(), 
        &payer.pubkey(), 
        &mint.pubkey(), 
        &spl_token::id()
    );

    let mint_to_ix = mint_to(
        &spl_token::id(), 
        &mint.pubkey(), 
        &payer_ata, 
        &payer.pubkey(), 
        &[], 
        1_000_000_000
    ).unwrap();

    let bh = client.get_latest_blockhash().unwrap();
    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, init_mint_ix, create_payer_ata_ix, mint_to_ix], 
        Some(&payer.pubkey()), 
        &[&payer, &mint], 
        bh
    );

    let sig1 = client.send_and_confirm_transaction(&tx).unwrap();
    
    println!("Mint + Supply: {sig1}");

    let recipient = Keypair::new();
    let recipient_ata = get_associated_token_address(
        &recipient.pubkey(),
        &mint.pubkey(),
    );
    let create_recipient_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &recipient.pubkey(),
        &mint.pubkey(),
        &spl_token::id(),
    );
    let transfer_ix = transfer_checked(
        &spl_token::id(), 
        &payer_ata, 
        &mint.pubkey(), 
        &recipient_ata, 
        &payer.pubkey(), 
        &[], 
        100_000_000,
        6,
    ).unwrap();

    let bh2 = client.get_latest_blockhash().unwrap();
    let tx2 = Transaction::new_signed_with_payer(
        &[create_recipient_ata_ix, transfer_ix], 
        Some(&payer.pubkey()), 
        &[&payer], 
        bh2,
    );
    let sig2 = client.send_and_confirm_transaction(&tx2).unwrap();
    println!("Transfer: {sig2} to {}", recipient.pubkey());

    let bal = client.get_token_account_balance(&recipient_ata).unwrap();
    assert_eq!(bal.amount, "100000000");
}
