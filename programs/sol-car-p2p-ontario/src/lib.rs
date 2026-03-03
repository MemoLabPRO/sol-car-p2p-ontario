use anchor_lang::prelude::*;

declare_id!("EQDnpsPH5W2TBATpoYdo7c3a5DKq7X12co14MVn7gB8B");

#[program]
pub mod sol_car_p2p_ontario {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        government_wallet: Pubkey,
        fee_basis_points: u16,
    ) -> Result<()> {
        let state = &mut ctx.accounts.program_state;
        state.authority = *ctx.accounts.authority.key;
        state.government_wallet = government_wallet;
        state.tax_basis_points = 1300;
        state.fee_basis_points = fee_basis_points;
        state.total_transactions = 0;
        state.bump = ctx.bumps.program_state;
        msg!("MTO Ontario: Sistema inicializado");
        Ok(())
    }

    pub fn register_vehicle(
        ctx: Context<RegisterVehicle>,
        vin: String,
        make: String,
        model: String,
        year: u16,
        color: String,
        has_safety: bool,
    ) -> Result<()> {
        require!(vin.len() == 17, CarError::InvalidVin);
        let vehicle = &mut ctx.accounts.vehicle;
        vehicle.vin = vin.clone();
        vehicle.make = make.clone();
        vehicle.model = model.clone();
        vehicle.year = year;
        vehicle.color = color;
        vehicle.owner = ctx.accounts.authority.key();
        vehicle.has_safety = has_safety;
        vehicle.is_stolen = false;
        vehicle.is_listed = false;
        vehicle.asking_price = 0;
        vehicle.transfer_count = 0;
        vehicle.last_sale_price = 0;
        vehicle.registered_at = Clock::get()?.unix_timestamp;
        vehicle.last_sold_at = 0;
        vehicle.bump = ctx.bumps.vehicle;
        msg!("Vehiculo registrado: {} {} {} VIN: {}", year, make, model, vin);
        Ok(())
    }

    pub fn update_safety(ctx: Context<AuthorityAction>, has_safety: bool) -> Result<()> {
        ctx.accounts.vehicle.has_safety = has_safety;
        msg!("Safety actualizado VIN: {}", ctx.accounts.vehicle.vin);
        Ok(())
    }

    pub fn flag_stolen(ctx: Context<AuthorityAction>, is_stolen: bool) -> Result<()> {
        ctx.accounts.vehicle.is_stolen = is_stolen;
        if is_stolen {
            msg!("ALERTA: VIN {} marcado como ROBADO", ctx.accounts.vehicle.vin);
        } else {
            msg!("VIN {} reporte de robo removido", ctx.accounts.vehicle.vin);
        }
        Ok(())
    }

    pub fn list_for_sale(ctx: Context<ListForSale>, price: u64) -> Result<()> {
        require!(price > 0, CarError::InvalidPrice);
        require!(!ctx.accounts.vehicle.is_stolen, CarError::CarIsStolen);
        require!(ctx.accounts.vehicle.owner == ctx.accounts.seller.key(), CarError::NotOwner);
        let vehicle = &mut ctx.accounts.vehicle;
        vehicle.is_listed = true;
        vehicle.asking_price = price;
        msg!("VIN {} en venta por {} lamports", vehicle.vin, price);
        Ok(())
    }

    pub fn execute_sale(ctx: Context<ExecuteSale>) -> Result<()> {
        let is_listed = ctx.accounts.vehicle.is_listed;
        let is_stolen = ctx.accounts.vehicle.is_stolen;
        let owner = ctx.accounts.vehicle.owner;
        let price = ctx.accounts.vehicle.asking_price;
        let tax_bps = ctx.accounts.program_state.tax_basis_points;
        let fee_bps = ctx.accounts.program_state.fee_basis_points;

        require!(is_listed, CarError::NotForSale);
        require!(!is_stolen, CarError::CarIsStolen);
        require!(owner == ctx.accounts.seller.key(), CarError::NotOwner);

        let tax = (price * tax_bps as u64) / 10000;
        let fee = (price * fee_bps as u64) / 10000;
        let buyer_total = price + tax + fee;

        // 1. Precio acordado al vendedor
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &ctx.accounts.seller.key(),
                price,
            ),
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.seller.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // 2. HST 13% a MTO Ontario
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &ctx.accounts.government.key(),
                tax,
            ),
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.government.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        // 3. Fee al protocolo
        anchor_lang::solana_program::program::invoke(
            &anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &ctx.accounts.fee_receiver.key(),
                fee,
            ),
            &[
                ctx.accounts.buyer.to_account_info(),
                ctx.accounts.fee_receiver.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let vehicle = &mut ctx.accounts.vehicle;
        vehicle.owner = ctx.accounts.buyer.key();
        vehicle.is_listed = false;
        vehicle.asking_price = 0;
        vehicle.transfer_count += 1;
        vehicle.last_sold_at = Clock::get()?.unix_timestamp;
        vehicle.last_sale_price = price;

        let state = &mut ctx.accounts.program_state;
        state.total_transactions += 1;

        msg!(
            "Transferencia completa | VIN: {} | Nuevo dueno: {} | HST: {} | Total comprador: {}",
            vehicle.vin,
            ctx.accounts.buyer.key(),
            tax,
            buyer_total
        );
        Ok(())
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        require!(ctx.accounts.vehicle.owner == ctx.accounts.seller.key(), CarError::NotOwner);
        let vehicle = &mut ctx.accounts.vehicle;
        vehicle.is_listed = false;
        vehicle.asking_price = 0;
        msg!("Listing cancelado VIN: {}", vehicle.vin);
        Ok(())
    }
}

// ─── STRUCTS ────────────────────────────────────────────────

#[account]
pub struct ProgramState {
    pub authority: Pubkey,         // 32
    pub government_wallet: Pubkey, // 32
    pub tax_basis_points: u16,     // 2
    pub fee_basis_points: u16,     // 2
    pub total_transactions: u64,   // 8
    pub bump: u8,                  // 1
}                                  // total: 77

#[account]
pub struct VehicleRecord {
    pub vin: String,           // 4 + 17 = 21
    pub make: String,          // 4 + 20 = 24
    pub model: String,         // 4 + 20 = 24
    pub year: u16,             // 2
    pub color: String,         // 4 + 15 = 19
    pub owner: Pubkey,         // 32
    pub has_safety: bool,      // 1
    pub is_stolen: bool,       // 1
    pub is_listed: bool,       // 1
    pub asking_price: u64,     // 8
    pub transfer_count: u16,   // 2
    pub last_sale_price: u64,  // 8
    pub registered_at: i64,    // 8
    pub last_sold_at: i64,     // 8
    pub bump: u8,              // 1
}                              // total: 160

// ─── CONTEXTOS ──────────────────────────────────────────────

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 2 + 2 + 8 + 1,
        seeds = [b"state"],
        bump
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(vin: String)]
pub struct RegisterVehicle<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 21 + 24 + 24 + 2 + 19 + 32 + 1 + 1 + 1 + 8 + 2 + 8 + 8 + 8 + 1,
        seeds = [b"vehicle", vin.as_bytes()],
        bump
    )]
    pub vehicle: Account<'info, VehicleRecord>,
    #[account(
        seeds = [b"state"],
        bump = program_state.bump,
        has_one = authority
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AuthorityAction<'info> {
    #[account(mut)]
    pub vehicle: Account<'info, VehicleRecord>,
    #[account(
        seeds = [b"state"],
        bump = program_state.bump,
        has_one = authority
    )]
    pub program_state: Account<'info, ProgramState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ListForSale<'info> {
    #[account(mut)]
    pub vehicle: Account<'info, VehicleRecord>,
    pub seller: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteSale<'info> {
    #[account(mut)]
    pub vehicle: Account<'info, VehicleRecord>,
    #[account(
        mut,
        seeds = [b"state"],
        bump = program_state.bump
    )]
    pub program_state: Account<'info, ProgramState>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    /// CHECK: Validado contra vehicle.owner
    #[account(
        mut,
        constraint = seller.key() == vehicle.owner @ CarError::NotOwner
    )]
    pub seller: AccountInfo<'info>,
    /// CHECK: Validado contra program_state.government_wallet
    #[account(
        mut,
        constraint = government.key() == program_state.government_wallet @ CarError::WrongGovernment
    )]
    pub government: AccountInfo<'info>,
    /// CHECK: Wallet que recibe el fee del protocolo
    #[account(mut)]
    pub fee_receiver: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut)]
    pub vehicle: Account<'info, VehicleRecord>,
    pub seller: Signer<'info>,
}

// ─── ERRORES ────────────────────────────────────────────────

#[error_code]
pub enum CarError {
    #[msg("El auto ya fue vendido.")]
    AlreadySold,
    #[msg("ADVERTENCIA: Vehiculo reportado como robado.")]
    CarIsStolen,
    #[msg("VIN debe tener exactamente 17 caracteres.")]
    InvalidVin,
    #[msg("El precio debe ser mayor a cero.")]
    InvalidPrice,
    #[msg("No eres el dueno de este vehiculo.")]
    NotOwner,
    #[msg("Este vehiculo no esta en venta.")]
    NotForSale,
    #[msg("La wallet del gobierno no coincide.")]
    WrongGovernment,
}
