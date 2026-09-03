use solana_client::rpc_client::RpcClient;

use solana_sdk::program_pack::Pack;
use solana_sdk::signer::Signer;
use solana_sdk::signer::keypair::{read_keypair_file,Keypair};
use solana_sdk::transaction::Transaction;

use solana_system_interface::instruction::create_account;

use spl_token::instruction::initialize_mint;
use spl_token::state::Mint;
use spl_token::instruction::mint_to;

use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;

fn main() {
    let client = RpcClient::new(
        "https://api.devnet.solana.com"
    );

    let payer = read_keypair_file(
        "devnet-wallet.json"
    ).expect("keypair not found");
    
    let mint = Keypair::new();

    let space = Mint::LEN;
    let rent = client
        .get_minimum_balance_for_rent_exemption(space)
        .expect("rent lookup failed");

    let create_ix = create_account(
        &payer.pubkey(),
        &mint.pubkey(),
        rent,
        space as u64,
        &spl_token::id(),
    );
    

    let init_ix = initialize_mint(
        &spl_token::id(),
        &mint.pubkey(),
        &payer.pubkey(),    
        Some(&payer.pubkey()), 
        6,                    
    )
    .expect("build initialize_mint failed");

    let blockhash = client.get_latest_blockhash().expect("blockhash failed");

    let tx = Transaction::new_signed_with_payer(
        &[create_ix, init_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        blockhash,
    );

    let sig = client.send_and_confirm_transaction(&tx).expect("send failed");

    println!("mint: {}", mint.pubkey());
    println!("tx:   {sig}");
    println!(
        "explorer: https://explorer.solana.com/address/{}?cluster=devnet",
        mint.pubkey()
    );

    let ata = get_associated_token_address(&payer.pubkey(), &mint.pubkey());

    let create_ata_ix = create_associated_token_account(
        &payer.pubkey(), 
        &payer.pubkey(),  
        &mint.pubkey(),  
        &spl_token::id(), 
    );

    let mint_to_ix = mint_to(
        &spl_token::id(),
        &mint.pubkey(),
        &ata,
        &payer.pubkey(), 
        &[],            
        1_000_000_000,
    )
    .expect("build mint_to failed");

    let blockhash2 = client
        .get_latest_blockhash()
        .expect("blockhash2 failed");

    let tx2 = Transaction::new_signed_with_payer(
        &[create_ata_ix, mint_to_ix],
        Some(&payer.pubkey()),
        &[&payer],
        blockhash2,
    );

    let sig2 = client.send_and_confirm_transaction(&tx2).expect("mint_to send failed");

    let bal = client.get_token_account_balance(&ata).expect("balance read failed");

    println!("ata:    {ata}");
    println!("minted: {} (tx {sig2})", bal.ui_amount_string);
}
