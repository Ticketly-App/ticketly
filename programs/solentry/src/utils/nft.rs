use anchor_lang::prelude::*;
use anchor_spl::metadata::{
    create_metadata_accounts_v3,
    update_metadata_accounts_v2,
    CreateMetadataAccountsV3,
    UpdateMetadataAccountsV2,
    Metadata,
    mpl_token_metadata::types::{DataV2, Creator},
};

pub fn create_ticket_metadata<'info>(
    metadata_program: &Program<'info, Metadata>,
    metadata_account: &AccountInfo<'info>,
    mint:             &AccountInfo<'info>,
    mint_authority:   &AccountInfo<'info>,   // ticket PDA
    payer:            &AccountInfo<'info>,
    system_program:   &AccountInfo<'info>,
    rent:             &AccountInfo<'info>,
    name:             String,
    symbol:           String,
    uri:              String,
    seller_fee_bps:   u16,
    organiser:        Pubkey,
    signer_seeds:     &[&[&[u8]]],
) -> Result<()> {
    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata:           metadata_account.clone(),
                mint:               mint.clone(),
                mint_authority:     mint_authority.clone(),
                update_authority:   mint_authority.clone(),
                payer:              payer.clone(),
                system_program:     system_program.clone(),
                rent:               rent.clone(),
            },
            signer_seeds,
        ),
        DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: seller_fee_bps,
            creators: Some(vec![Creator {
                address:  organiser,
                verified: false, // organiser must call verify separately
                share:    100,
            }]),
            collection: None,
            uses:       None,
        },
        true,  // is_mutable — allows POAP upgrade post-event
        true,  // update_authority_is_signer
        None,  // collection_details
    )
}

pub fn freeze_ticket_metadata<'info>(
    metadata_program: &Program<'info, Metadata>,
    metadata_account: &AccountInfo<'info>,
    update_authority: &AccountInfo<'info>,   // ticket PDA
    signer_seeds:     &[&[&[u8]]],
) -> Result<()> {
    update_metadata_accounts_v2(
        CpiContext::new_with_signer(
            metadata_program.to_account_info(),
            UpdateMetadataAccountsV2 {
                metadata:         metadata_account.clone(),
                update_authority: update_authority.clone(),
            },
            signer_seeds,
        ),
        None,       // new update authority
        None,       // data (no change)
        Some(false), // primary_sale_happened — mark it sold
        Some(false), // is_mutable — freeze after check-in
    )
}