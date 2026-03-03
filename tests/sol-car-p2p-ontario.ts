import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolCarP2pOntario } from "../target/types/sol_car_p2p_ontario";
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";

describe("sol-car-p2p-ontario", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolCarP2pOntario as Program<SolCarP2pOntario>;

  // Wallets simuladas
  const authority = provider.wallet; // MTO Ontario (nosotros, el deployer)
  const seller = anchor.web3.Keypair.generate(); // Vendedor
  const buyer = anchor.web3.Keypair.generate(); // Comprador
  const government = anchor.web3.Keypair.generate(); // Wallet del gobierno Ontario
  const feeReceiver = anchor.web3.Keypair.generate(); // Wallet del protocolo

  // VIN real de una Tacoma 2022
  const VIN = "1HGBH41JXMN109186";

  // PDAs
  let programStatePda: PublicKey;
  let vehiclePda: PublicKey;

before(async () => {
    // Derivar PDAs
    [programStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );

    [vehiclePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vehicle"), Buffer.from(VIN)],
      program.programId
    );

    // Fondear seller y buyer desde nuestra wallet que ya tiene SOL
    const transferToSeller = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: authority.publicKey,
        toPubkey: seller.publicKey,
        lamports: 1 * LAMPORTS_PER_SOL,
      })
    );
    await provider.sendAndConfirm(transferToSeller);

    const transferToBuyer = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: authority.publicKey,
        toPubkey: buyer.publicKey,
        lamports: 3 * LAMPORTS_PER_SOL,
      })
    );
    await provider.sendAndConfirm(transferToBuyer);

    console.log("Seller fondeado:", seller.publicKey.toString());
    console.log("Buyer fondeado:", buyer.publicKey.toString());
  });

  // ─── TEST 1 ──────────────────────────────────────────────
  it("MTO Ontario inicializa el sistema", async () => {
    await program.methods
      .initialize(government.publicKey, 50) // 50 = 0.5% fee
      .accounts({
        programState: programStatePda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const state = await program.account.programState.fetch(programStatePda);

    assert.equal(state.taxBasisPoints, 1300, "HST debe ser 13%");
    assert.equal(state.feeBasisPoints, 50, "Fee debe ser 0.5%");
    assert.equal(
      state.governmentWallet.toString(),
      government.publicKey.toString(),
      "Gobierno registrado correctamente"
    );

    console.log("✅ Sistema inicializado");
    console.log("   Gobierno:", government.publicKey.toString());
  });

  // ─── TEST 2 ──────────────────────────────────────────────
  it("MTO Ontario registra un vehiculo", async () => {
    await program.methods
      .registerVehicle(VIN, "Toyota", "Tacoma", 2022, "White", true)
      .accounts({
        vehicle: vehiclePda,
        programState: programStatePda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const vehicle = await program.account.vehicleRecord.fetch(vehiclePda);

    assert.equal(vehicle.vin, VIN);
    assert.equal(vehicle.make, "Toyota");
    assert.equal(vehicle.model, "Tacoma");
    assert.equal(vehicle.year, 2022);
    assert.equal(vehicle.hasSafety, true);
    assert.equal(vehicle.isStolen, false);
    assert.equal(vehicle.isListed, false);

    console.log("✅ Vehiculo registrado:", vehicle.make, vehicle.model, vehicle.year);
    console.log("   Safety certificate:", vehicle.hasSafety);
    console.log("   Dueno inicial:", vehicle.owner.toString());
  });

  // ─── TEST 3 ──────────────────────────────────────────────
  it("No se puede vender un auto robado", async () => {
    // Marcar como robado
    await program.methods
      .flagStolen(true)
      .accounts({
        vehicle: vehiclePda,
        programState: programStatePda,
        authority: authority.publicKey,
      })
      .rpc();

    let vehicle = await program.account.vehicleRecord.fetch(vehiclePda);
    assert.equal(vehicle.isStolen, true, "Debe estar marcado como robado");

    // Intentar listar — debe fallar
    try {
      await program.methods
        .listForSale(new anchor.BN(1 * LAMPORTS_PER_SOL))
        .accounts({
          vehicle: vehiclePda,
          seller: authority.publicKey,
        })
        .rpc();
      assert.fail("Debia fallar por auto robado");
    } catch (err) {
      assert.include(err.message, "CarIsStolen");
      console.log("✅ Correcto: no se puede vender auto robado");
    }

    // Limpiar reporte
    await program.methods
      .flagStolen(false)
      .accounts({
        vehicle: vehiclePda,
        programState: programStatePda,
        authority: authority.publicKey,
      })
      .rpc();

    vehicle = await program.account.vehicleRecord.fetch(vehiclePda);
    assert.equal(vehicle.isStolen, false, "Reporte limpiado");
    console.log("✅ Reporte de robo removido");
  });

  // ─── TEST 4 ──────────────────────────────────────────────
  it("Vendedor lista el auto para venta", async () => {
    const precioVenta = 1 * LAMPORTS_PER_SOL; // 1 SOL

    await program.methods
      .listForSale(new anchor.BN(precioVenta))
      .accounts({
        vehicle: vehiclePda,
        seller: authority.publicKey,
      })
      .rpc();

    const vehicle = await program.account.vehicleRecord.fetch(vehiclePda);

    assert.equal(vehicle.isListed, true, "Debe estar listado");
    assert.equal(
      vehicle.askingPrice.toNumber(),
      precioVenta,
      "Precio correcto"
    );

    console.log("✅ Auto listado para venta");
    console.log("   Precio:", precioVenta / LAMPORTS_PER_SOL, "SOL");
  });

  // ─── TEST 5 ──────────────────────────────────────────────
  it("Comprador ejecuta la compra y HST va al gobierno automaticamente", async () => {
    const price = 1 * LAMPORTS_PER_SOL;
    const expectedTax = Math.floor(price * 1300 / 10000); // 13%
    const expectedFee = Math.floor(price * 50 / 10000);   // 0.5%

    // Balances antes
    const govBalanceBefore = await provider.connection.getBalance(government.publicKey);
    const sellerBalanceBefore = await provider.connection.getBalance(authority.publicKey);

    await program.methods
      .executeSale()
      .accounts({
        vehicle: vehiclePda,
        programState: programStatePda,
        buyer: buyer.publicKey,
        seller: authority.publicKey,
        government: government.publicKey,
        feeReceiver: feeReceiver.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([buyer])
      .rpc();

    // Balances después
    const govBalanceAfter = await provider.connection.getBalance(government.publicKey);
    const vehicle = await program.account.vehicleRecord.fetch(vehiclePda);
    const state = await program.account.programState.fetch(programStatePda);

    // Verificaciones
    assert.equal(
      vehicle.owner.toString(),
      buyer.publicKey.toString(),
      "El comprador es el nuevo dueno"
    );
    assert.equal(vehicle.isListed, false, "Ya no esta en venta");
    assert.equal(vehicle.transferCount, 1, "Una transferencia registrada");
    assert.equal(
      govBalanceAfter - govBalanceBefore,
      expectedTax,
      "Gobierno recibio exactamente el 13% HST"
    );
    assert.equal(state.totalTransactions.toNumber(), 1, "Una transaccion total");

    console.log("✅ Compraventa exitosa");
    console.log("   Nuevo dueno:", vehicle.owner.toString());
    console.log("   HST pagado al gobierno:", expectedTax / LAMPORTS_PER_SOL, "SOL");
    console.log("   Fee pagado:", expectedFee / LAMPORTS_PER_SOL, "SOL");
    console.log("   Transferencias del vehiculo:", vehicle.transferCount);
  });
});