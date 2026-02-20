import * as anchor from "@coral-xyz/anchor";
import type { Program } from "@coral-xyz/anchor";
import BN from "bn.js";
import {
  Keypair,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";
import type { Ticketly } from "../target/types/ticketly";

// Constants 
const MPL_METADATA = new PublicKey(
  "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
);

// PDA helpers

const pda = (seeds: (Buffer | Uint8Array)[], prog: PublicKey) =>
  PublicKey.findProgramAddressSync(seeds, prog);

const eventPda = (auth: PublicKey, id: BN, prog: PublicKey) =>
  pda([Buffer.from("event"), auth.toBuffer(), id.toArrayLike(Buffer, "le", 8)], prog);

const ticketPda = (ev: PublicKey, num: BN, prog: PublicKey) =>
  pda([Buffer.from("ticket"), ev.toBuffer(), num.toArrayLike(Buffer, "le", 8)], prog);

const mintPda = (ticket: PublicKey, prog: PublicKey) =>
  pda([Buffer.from("ticket_mint"), ticket.toBuffer()], prog);

const listingPda = (ticket: PublicKey, prog: PublicKey) =>
  pda([Buffer.from("listing"), ticket.toBuffer()], prog);

const metadataPda = (mint: PublicKey) =>
  pda([Buffer.from("metadata"), MPL_METADATA.toBuffer(), mint.toBuffer()], MPL_METADATA);

const organizerPda = (auth: PublicKey, prog: PublicKey) =>
  pda([Buffer.from("organizer"), auth.toBuffer()], prog);

const platformPda = (prog: PublicKey) =>
  pda([Buffer.from("platform")], prog);

const whitelistPda = (ev: PublicKey, wallet: PublicKey, prog: PublicKey) =>
  pda([Buffer.from("whitelist"), ev.toBuffer(), wallet.toBuffer()], prog);

const poapPda = (ticket: PublicKey, prog: PublicKey) =>
  pda([Buffer.from("poap"), ticket.toBuffer()], prog);

const poapMintPda = (ticket: PublicKey, prog: PublicKey) =>
  pda([Buffer.from("poap_mint"), ticket.toBuffer()], prog);

// Utility 

const now = () => Math.floor(Date.now() / 1000);

async function chainNow(conn: anchor.web3.Connection) {
  const slot = await conn.getSlot("confirmed");
  const bt = await conn.getBlockTime(slot);
  return bt ?? now();
}

async function airdrop(conn: anchor.web3.Connection, pk: PublicKey, sol = 20) {
  const sig = await conn.requestAirdrop(pk, sol * LAMPORTS_PER_SOL);
  await conn.confirmTransaction(sig, "confirmed");
}

function baseTiers(override: Partial<any> = {}) {
  return [
    {
      tierType:   { generalAdmission: {} },
      price:      new BN(0.5 * LAMPORTS_PER_SOL),
      supply:     50,
      minted:     0,
      checkedIn:  0,
      isOnSale:   true,
      saleStart:  new BN(0),
      saleEnd:    new BN(0),
      ...override,
    },
    {
      tierType:   { vip: {} },
      price:      new BN(2 * LAMPORTS_PER_SOL),
      supply:     10,
      minted:     0,
      checkedIn:  0,
      isOnSale:   true,
      saleStart:  new BN(0),
      saleEnd:    new BN(0),
    },
  ];
}

function baseEventParams(eventId: BN, override: Partial<any> = {}) {
  return {
    eventId,
    name:            "Onchain Luma Fest",
    description:     "The best web3 event",
    venue:           "Austin, TX",
    metadataUri:     "https://arweave.net/event",
    symbol:          "LUMA",
    gps:             { latMicro: 30_267_000, lngMicro: -97_743_000 },
    eventStart:      new BN(now() + 3600),
    eventEnd:        new BN(now() + 3600 + 28800),
    ticketTiers:     baseTiers(),
    resaleAllowed:   true,
    maxResalePrice:  new BN(5 * LAMPORTS_PER_SOL),
    royaltyBps:      500,
    whitelistGated:  false,
    poapEnabled:     false,
    poapMetadataUri: "",
    ...override,
  };
}

// Test suite 

describe("ticketly — complete suite", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program   = anchor.workspace.ticketly as Program<ticketly>;
  const conn      = provider.connection;
  const progId    = program.programId;

  const organiser  = Keypair.generate();
  const buyer1     = Keypair.generate();
  const buyer2     = Keypair.generate();
  const operator   = Keypair.generate();
  const platformAdmin = (provider.wallet as anchor.Wallet & { payer: Keypair }).payer;
  let platformAdminMismatch = false;

  const EVENT_ID  = new BN(1);
  let   eventKey: PublicKey;
  let   eventBump: number;

  before(async () => {
    await Promise.all([
      airdrop(conn, organiser.publicKey),
      airdrop(conn, buyer1.publicKey),
      airdrop(conn, buyer2.publicKey),
      airdrop(conn, operator.publicKey),
    ]);
    [eventKey, eventBump] = eventPda(organiser.publicKey, EVENT_ID, progId);
  });

  // Platform 

  describe("platform", () => {
    it("initialises platform config", async function () {
      const [platKey] = platformPda(progId);

      const existing = await program.account.platformConfig.fetchNullable(platKey);
      if (existing && existing.admin.toBase58() !== platformAdmin.publicKey.toBase58()) {
        platformAdminMismatch = true;
        expect(existing.protocolFeeBps).to.be.at.most(1000);
        return;
      }

      if (!existing) {
        await program.methods
          .initPlatform(100) // 1% fee
          .accounts({
            platformConfig: platKey,
            admin:          platformAdmin.publicKey,
            systemProgram:  SystemProgram.programId,
          })
          .signers([platformAdmin])
          .rpc();
      } else {
        await program.methods
          .updatePlatform(100, null, null)
          .accounts({ platformConfig: platKey, admin: platformAdmin.publicKey })
          .signers([platformAdmin])
          .rpc();
      }

      const cfg = await program.account.platformConfig.fetch(platKey);
      expect(cfg.protocolFeeBps).to.equal(100);
      expect(cfg.creationPaused).to.be.false;
    });

    it("updates fee receiver", async () => {

      const [platKey] = platformPda(progId);
      if (platformAdminMismatch) {
        try {
          await program.methods
            .updatePlatform(null, organiser.publicKey, null)
            .accounts({ platformConfig: platKey, admin: platformAdmin.publicKey })
            .signers([platformAdmin])
            .rpc();
          expect.fail("should throw");
        } catch (e: any) {
          expect(e?.error?.errorCode?.code).to.equal("NotEventAuthority");
        }
        return;
      }

      await program.methods
        .updatePlatform(null, organiser.publicKey, null)
        .accounts({ platformConfig: platKey, admin: platformAdmin.publicKey })
        .signers([platformAdmin])
        .rpc();

      const cfg = await program.account.platformConfig.fetch(platKey);
      expect(cfg.feeReceiver.toBase58()).to.equal(organiser.publicKey.toBase58());
    });

    it("rejects fee > 1000 bps", async () => {

      const [platKey] = platformPda(progId);
      try {
        await program.methods
          .updatePlatform(1001, null, null)
          .accounts({ platformConfig: platKey, admin: platformAdmin.publicKey })
          .signers([platformAdmin])
          .rpc();
        expect.fail("should throw");
      } catch (e: any) {
        const code = e?.error?.errorCode?.code;
        if (platformAdminMismatch) {
          expect(code).to.equal("NotEventAuthority");
        } else {
          expect(code).to.equal("InvalidRoyalty");
        }
      }
    });
  });

  // Organiser profile 

  describe("organizer_profile", () => {
    it("creates a profile", async () => {
      const [profKey] = organizerPda(organiser.publicKey, progId);
      await program.methods
        .initOrganizer({ name: "Luma Labs", website: "https://luma.xyz", logoUri: "ipfs://logo" })
        .accounts({ organizerProfile: profKey, authority: organiser.publicKey, systemProgram: SystemProgram.programId })
        .signers([organiser])
        .rpc();

      const p = await program.account.organizerProfile.fetch(profKey);
      expect(p.name).to.equal("Luma Labs");
      expect(p.totalEvents).to.equal(0);
    });

    it("updates the profile", async () => {
      const [profKey] = organizerPda(organiser.publicKey, progId);
      await program.methods
        .updateOrganizer({ name: "Luma Labs v2", website: "https://luma.xyz", logoUri: "ipfs://logo2" })
        .accounts({ organizerProfile: profKey, authority: organiser.publicKey })
        .signers([organiser])
        .rpc();

      const p = await program.account.organizerProfile.fetch(profKey);
      expect(p.name).to.equal("Luma Labs v2");
    });
  });

  // Event lifecycle 

  describe("create_event", () => {
    it("creates event and increments organiser profile total_events", async () => {
      const [profKey] = organizerPda(organiser.publicKey, progId);

      await program.methods
        .createEvent(baseEventParams(EVENT_ID))
        .accounts({
          event:             eventKey,
          organizerProfile:  profKey,
          authority:         organiser.publicKey,
          systemProgram:     SystemProgram.programId,
        })
        .signers([organiser])
        .rpc();

      const ev   = await program.account.eventAccount.fetch(eventKey);
      const prof = await program.account.organizerProfile.fetch(profKey);
      expect(ev.name).to.equal("Onchain Luma Fest");
      expect(ev.isActive).to.be.true;
      expect(ev.whitelistGated).to.be.false;
      expect(prof.totalEvents).to.equal(1);
    });

    it("rejects royalty > 2000 bps", async () => {
      const id = new BN(9001);
      const [pda_] = eventPda(organiser.publicKey, id, progId);
      try {
        await program.methods
          .createEvent(baseEventParams(id, { royaltyBps: 2001 }))
          .accounts({ event: pda_, organizerProfile: null, authority: organiser.publicKey, systemProgram: SystemProgram.programId })
          .signers([organiser])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e.error.errorCode.code).to.equal("InvalidRoyalty");
      }
    });

    it("rejects name > 50 chars", async () => {
      const id = new BN(9002);
      const [pda_] = eventPda(organiser.publicKey, id, progId);
      try {
        await program.methods
          .createEvent(baseEventParams(id, { name: "A".repeat(51) }))
          .accounts({ event: pda_, organizerProfile: null, authority: organiser.publicKey, systemProgram: SystemProgram.programId })
          .signers([organiser])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e.error.errorCode.code).to.equal("NameTooLong");
      }
    });
  });

  describe("update_event", () => {
    it("updates venue", async () => {
      await program.methods
        .updateEvent({
          name: null, description: null, venue: "Miami, FL",
          metadataUri: null, eventStart: null, eventEnd: null,
          resaleAllowed: null, maxResalePrice: null, royaltyBps: null,
        })
        .accounts({ event: eventKey, authority: organiser.publicKey })
        .signers([organiser])
        .rpc();

      const ev = await program.account.eventAccount.fetch(eventKey);
      expect(ev.venue).to.equal("Miami, FL");
    });

    it("rejects non-authority update", async () => {
      try {
        await program.methods
          .updateEvent({ name: "Hacked", description: null, venue: null, metadataUri: null, eventStart: null, eventEnd: null, resaleAllowed: null, maxResalePrice: null, royaltyBps: null })
          .accounts({ event: eventKey, authority: buyer1.publicKey })
          .signers([buyer1])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e.error.errorCode.code).to.equal("NotEventAuthority");
      }
    });
  });

  // Gate operators 

  describe("gate_operators", () => {
    it("adds and removes an operator", async () => {
      await program.methods
        .addOperator(operator.publicKey)
        .accounts({ event: eventKey, authority: organiser.publicKey })
        .signers([organiser])
        .rpc();

      let ev = await program.account.eventAccount.fetch(eventKey);
      expect(ev.gateOperators.map((k: PublicKey) => k.toBase58()))
        .to.include(operator.publicKey.toBase58());

      await program.methods
        .removeOperator(operator.publicKey)
        .accounts({ event: eventKey, authority: organiser.publicKey })
        .signers([organiser])
        .rpc();

      ev = await program.account.eventAccount.fetch(eventKey);
      expect(ev.gateOperators.map((k: PublicKey) => k.toBase58()))
        .not.to.include(operator.publicKey.toBase58());
    });

    it("rejects duplicate operator add", async () => {
      await program.methods
        .addOperator(operator.publicKey)
        .accounts({ event: eventKey, authority: organiser.publicKey })
        .signers([organiser])
        .rpc();

      try {
        await program.methods
          .addOperator(operator.publicKey)
          .accounts({ event: eventKey, authority: organiser.publicKey })
          .signers([organiser])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e.error.errorCode.code).to.equal("OperatorAlreadyAdded");
      }
    });

    it("rejects operator add by non-authority", async () => {
      try {
        await program.methods
          .addOperator(Keypair.generate().publicKey)
          .accounts({ event: eventKey, authority: buyer1.publicKey })
          .signers([buyer1])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e?.error?.errorCode?.code).to.equal("NotEventAuthority");
      }
    });

    it("enforces max 10 operators", async () => {
      const existing = await program.account.eventAccount.fetch(eventKey);
      const current = existing.gateOperators.length;
      for (let i = current; i < 10; i++) {
        await program.methods
          .addOperator(Keypair.generate().publicKey)
          .accounts({ event: eventKey, authority: organiser.publicKey })
          .signers([organiser])
          .rpc();
      }

      try {
        await program.methods
          .addOperator(Keypair.generate().publicKey)
          .accounts({ event: eventKey, authority: organiser.publicKey })
          .signers([organiser])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e?.error?.errorCode?.code).to.equal("TooManyOperators");
      }
    });
  });

  // Whitelist

  describe("whitelist", () => {
    const WL_EVENT_ID = new BN(77);
    let   wlEventKey: PublicKey;

    before(async () => {
      [wlEventKey] = eventPda(organiser.publicKey, WL_EVENT_ID, progId);
      // Create whitelist-gated event
      await program.methods
        .createEvent(baseEventParams(WL_EVENT_ID, { whitelistGated: true }))
        .accounts({ event: wlEventKey, organizerProfile: null, authority: organiser.publicKey, systemProgram: SystemProgram.programId })
        .signers([organiser])
        .rpc();
    });

    it("adds a whitelist entry", async () => {
      const [wlKey] = whitelistPda(wlEventKey, buyer1.publicKey, progId);
      await program.methods
        .addWhitelistEntry(buyer1.publicKey, 2)
        .accounts({
          event:          wlEventKey,
          whitelistEntry: wlKey,
          authority:      organiser.publicKey,
          systemProgram:  SystemProgram.programId,
        })
        .signers([organiser])
        .rpc();

      const wl = await program.account.whitelistEntry.fetch(wlKey);
      expect(wl.allocation).to.equal(2);
      expect(wl.purchased).to.equal(0);
    });

    it("removes a whitelist entry (rent returned)", async () => {
      const [wlKey] = whitelistPda(wlEventKey, buyer1.publicKey, progId);
      const balBefore = await conn.getBalance(organiser.publicKey);

      await program.methods
        .removeWhitelistEntry()
        .accounts({
          event: wlEventKey, whitelistEntry: wlKey, authority: organiser.publicKey,
        })
        .signers([organiser])
        .rpc();

      const balAfter = await conn.getBalance(organiser.publicKey);
      expect(balAfter).to.be.greaterThan(balBefore - 5000); // rent returned
    });

    it("rejects whitelist entry when event is not whitelist-gated", async () => {
      const id = new BN(78);
      const [eventKeyNoWl] = eventPda(organiser.publicKey, id, progId);
      await program.methods
        .createEvent(baseEventParams(id, { whitelistGated: false }))
        .accounts({
          event: eventKeyNoWl,
          organizerProfile: null,
          authority: organiser.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([organiser])
        .rpc();

      const [wlKey] = whitelistPda(eventKeyNoWl, buyer2.publicKey, progId);
      try {
        await program.methods
          .addWhitelistEntry(buyer2.publicKey, 1)
          .accounts({
            event: eventKeyNoWl,
            whitelistEntry: wlKey,
            authority: organiser.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([organiser])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e?.error?.errorCode?.code).to.equal("WhitelistNotEnabled");
      }
    });
  });

  // Ticket minting 

  describe("mint_ticket", () => {
    let ticket0Key: PublicKey;
    let mint0Key:   PublicKey;
    let ata0:       PublicKey;

    it("mints GA ticket to buyer1 and collects payment", async () => {
      const num = (await program.account.eventAccount.fetch(eventKey)).totalMinted;
      [ticket0Key] = ticketPda(eventKey, num, progId);
      [mint0Key]   = mintPda(ticket0Key, progId);
      ata0         = await getAssociatedTokenAddress(mint0Key, buyer1.publicKey);
      const [meta] = metadataPda(mint0Key);

      const balBefore = await conn.getBalance(eventKey);

      await program.methods
        .mintTicket({ tierIndex: 0, metadataUri: "https://arweave.net/t/0" })
        .accounts({
          event: eventKey, ticket: ticket0Key, mint: mint0Key,
          recipientAta: ata0, metadataAccount: meta,
          whitelistEntry: null,
          recipient: buyer1.publicKey, payer: buyer1.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenMetadataProgram: MPL_METADATA,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([buyer1])
        .rpc();

      const ev  = await program.account.eventAccount.fetch(eventKey);
      const tkt = await program.account.ticketAccount.fetch(ticket0Key);
      const tok = await getAccount(conn, ata0);

      expect(ev.totalMinted.toNumber()).to.equal(1);
      expect(ev.totalRevenue.toNumber()).to.equal(0.5 * LAMPORTS_PER_SOL);
      expect(tkt.owner.toBase58()).to.equal(buyer1.publicKey.toBase58());
      expect(tkt.isCheckedIn).to.be.false;
      expect(tkt.resaleCount).to.equal(0);
      expect(BigInt(tok.amount)).to.equal(1n);

      const balAfter = await conn.getBalance(eventKey);
      expect(balAfter - balBefore).to.equal(0.5 * LAMPORTS_PER_SOL);
    });

    it("rejects invalid tier index", async () => {
      const num = (await program.account.eventAccount.fetch(eventKey)).totalMinted;
      const [tKey] = ticketPda(eventKey, num, progId);
      const [mKey] = mintPda(tKey, progId);
      const ata    = await getAssociatedTokenAddress(mKey, buyer1.publicKey);
      const [meta] = metadataPda(mKey);

      try {
        await program.methods
          .mintTicket({ tierIndex: 99, metadataUri: "x" })
          .accounts({
            event: eventKey, ticket: tKey, mint: mKey, recipientAta: ata,
            metadataAccount: meta, whitelistEntry: null,
            recipient: buyer1.publicKey, payer: buyer1.publicKey,
            systemProgram: SystemProgram.programId, tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenMetadataProgram: MPL_METADATA, rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([buyer1])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e.error.errorCode.code).to.equal("InvalidTierIndex");
      }
    });

    it("requires whitelist entry on whitelist-gated events", async () => {
      const id = new BN(79);
      const [wlEventKey] = eventPda(organiser.publicKey, id, progId);

      await program.methods
        .createEvent(baseEventParams(id, { whitelistGated: true }))
        .accounts({
          event: wlEventKey,
          organizerProfile: null,
          authority: organiser.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([organiser])
        .rpc();

      const num = (await program.account.eventAccount.fetch(wlEventKey)).totalMinted;
      const [tKey] = ticketPda(wlEventKey, num, progId);
      const [mKey] = mintPda(tKey, progId);
      const ata = await getAssociatedTokenAddress(mKey, buyer2.publicKey);
      const [meta] = metadataPda(mKey);

      try {
        await program.methods
          .mintTicket({ tierIndex: 0, metadataUri: "https://arweave.net/no-wl" })
          .accounts({
            event: wlEventKey,
            ticket: tKey,
            mint: mKey,
            recipientAta: ata,
            metadataAccount: meta,
            whitelistEntry: null,
            recipient: buyer2.publicKey,
            payer: buyer2.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            tokenMetadataProgram: MPL_METADATA,
            rent: SYSVAR_RENT_PUBKEY,
          })
          .signers([buyer2])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e?.error?.errorCode?.code).to.equal("WhitelistEntryRequired");
      }
    });
  });

  // Transfer ticket 

  describe("transfer_ticket", () => {
    it("transfers ticket #0 from buyer1 to buyer2", async () => {
      const [ticket0Key] = ticketPda(eventKey, new BN(0), progId);
      const [mint0Key]   = mintPda(ticket0Key, progId);
      const sender_ata   = await getAssociatedTokenAddress(mint0Key, buyer1.publicKey);
      const recip_ata    = await getAssociatedTokenAddress(mint0Key, buyer2.publicKey);

      await program.methods
        .transferTicket()
        .accounts({
          event: eventKey, ticket: ticket0Key, mint: mint0Key,
          senderAta: sender_ata, recipientAta: recip_ata,
          sender: buyer1.publicKey, recipient: buyer2.publicKey,
          systemProgram: SystemProgram.programId, tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([buyer1])
        .rpc();

      const tkt = await program.account.ticketAccount.fetch(ticket0Key);
      expect(tkt.owner.toBase58()).to.equal(buyer2.publicKey.toBase58());
      expect(tkt.transferCount).to.equal(1);

      const recipTok = await getAccount(conn, recip_ata);
      expect(BigInt(recipTok.amount)).to.equal(1n);
    });
  });

  // Check-in 

  describe("check_in", () => {
    // Create a fresh event that has already started so timing guards pass
    const CI_EVENT_ID = new BN(100);
    let   ciEventKey: PublicKey;
    let   ciTicketKey: PublicKey;
    let   ciMintKey:   PublicKey;
    let   ciAta:       PublicKey;

    before(async () => {
      [ciEventKey] = eventPda(organiser.publicKey, CI_EVENT_ID, progId);
      const t = await chainNow(conn);

      // create_event enforces start in the future
      await program.methods
        .createEvent(baseEventParams(CI_EVENT_ID, {
          eventStart: new BN(t + 120),
          eventEnd:   new BN(t + 7200),
        }))
        .accounts({ event: ciEventKey, organizerProfile: null, authority: organiser.publicKey, systemProgram: SystemProgram.programId })
        .signers([organiser])
        .rpc();

      // then adjust to active check-in window
      await program.methods
        .updateEvent({
          name: null,
          description: null,
          venue: null,
          metadataUri: null,
          eventStart: new BN(t - 7200),
          eventEnd: new BN(t + 3600),
          resaleAllowed: null,
          maxResalePrice: null,
          royaltyBps: null,
        })
        .accounts({ event: ciEventKey, authority: organiser.publicKey })
        .signers([organiser])
        .rpc();

      // Mint ticket to buyer1
      const num = (await program.account.eventAccount.fetch(ciEventKey)).totalMinted;
      [ciTicketKey] = ticketPda(ciEventKey, num, progId);
      [ciMintKey]   = mintPda(ciTicketKey, progId);
      ciAta         = await getAssociatedTokenAddress(ciMintKey, buyer1.publicKey);
      const [meta]  = metadataPda(ciMintKey);

      await program.methods
        .mintTicket({ tierIndex: 0, metadataUri: "https://arweave.net/ci" })
        .accounts({
          event: ciEventKey, ticket: ciTicketKey, mint: ciMintKey,
          recipientAta: ciAta, metadataAccount: meta, whitelistEntry: null,
          recipient: buyer1.publicKey, payer: buyer1.publicKey,
          systemProgram: SystemProgram.programId, tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenMetadataProgram: MPL_METADATA, rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([buyer1])
        .rpc();
    });

    it("organiser can check in attendee", async () => {
      const [meta] = metadataPda(ciMintKey);

      await program.methods
        .checkInTicket()
        .accounts({
          event: ciEventKey, ticket: ciTicketKey,
          attendeeAta: ciAta, metadataAccount: meta,
          attendee: buyer1.publicKey, gateOperator: organiser.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          tokenMetadataProgram: MPL_METADATA,
        })
        .signers([organiser])
        .rpc();

      const tkt = await program.account.ticketAccount.fetch(ciTicketKey);
      expect(tkt.isCheckedIn).to.be.true;
      expect(tkt.checkedInBy.toBase58()).to.equal(organiser.publicKey.toBase58());
    });

    it("rejects double check-in", async () => {
      const [meta] = metadataPda(ciMintKey);
      try {
        await program.methods
          .checkInTicket()
          .accounts({
            event: ciEventKey, ticket: ciTicketKey,
            attendeeAta: ciAta, metadataAccount: meta,
            attendee: buyer1.publicKey, gateOperator: organiser.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID, tokenMetadataProgram: MPL_METADATA,
          })
          .signers([organiser])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e?.error?.errorCode?.code).to.equal("AlreadyCheckedIn");
      }
    });

    it("rejects check-in by non-operator", async () => {
      // Mint a second ticket
      const num = (await program.account.eventAccount.fetch(ciEventKey)).totalMinted;
      const [tKey] = ticketPda(ciEventKey, num, progId);
      const [mKey] = mintPda(tKey, progId);
      const ata    = await getAssociatedTokenAddress(mKey, buyer2.publicKey);
      const [meta] = metadataPda(mKey);

      await program.methods
        .mintTicket({ tierIndex: 0, metadataUri: "https://arweave.net/ci2" })
        .accounts({
          event: ciEventKey, ticket: tKey, mint: mKey,
          recipientAta: ata, metadataAccount: meta, whitelistEntry: null,
          recipient: buyer2.publicKey, payer: buyer2.publicKey,
          systemProgram: SystemProgram.programId, tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenMetadataProgram: MPL_METADATA, rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([buyer2])
        .rpc();

      try {
        await program.methods
          .checkInTicket()
          .accounts({
            event: ciEventKey, ticket: tKey, attendeeAta: ata,
            metadataAccount: meta, attendee: buyer2.publicKey,
            gateOperator: buyer1.publicKey,  // not an operator
            tokenProgram: TOKEN_PROGRAM_ID, tokenMetadataProgram: MPL_METADATA,
          })
          .signers([buyer1])
          .rpc();
        expect.fail();
      } catch (e: any) {
        expect(e?.error?.errorCode?.code).to.equal("NotGateOperator");
      }
    });
  });

  // Revenue withdrawal 

  describe("withdraw_revenue", () => {
    it("organiser can withdraw accumulated lamports", async () => {
      const balBefore = await conn.getBalance(organiser.publicKey);

      await program.methods
        .withdrawRevenue(null) // drain all
        .accounts({ event: eventKey, authority: organiser.publicKey })
        .signers([organiser])
        .rpc();

      const balAfter = await conn.getBalance(organiser.publicKey);
      expect(balAfter).to.be.greaterThan(balBefore);
    });
  });

  // Marketplace 

  describe("marketplace", () => {
    // ticket0 is with buyer2 after the transfer test above
    let   ticketKey: PublicKey;
    let   mintKey:   PublicKey;
    let   listingKey: PublicKey;
    let   sellerAta:  PublicKey;
    let   escrowAta:  PublicKey;
    let   buyerAta:   PublicKey;
    const PRICE = new BN(1 * LAMPORTS_PER_SOL);

    before(async () => {
      [ticketKey]  = ticketPda(eventKey, new BN(0), progId);
      [mintKey]    = mintPda(ticketKey, progId);
      [listingKey] = listingPda(ticketKey, progId);
      sellerAta    = await getAssociatedTokenAddress(mintKey, buyer2.publicKey);
      escrowAta    = await getAssociatedTokenAddress(mintKey, listingKey, true);
      buyerAta     = await getAssociatedTokenAddress(mintKey, buyer1.publicKey);
    });

    it("lists ticket for 1 SOL", async () => {
      await program.methods
        .listTicket(PRICE)
        .accounts({
          event: eventKey, ticket: ticketKey, listing: listingKey, mint: mintKey,
          sellerAta, escrowAta, seller: buyer2.publicKey,
          systemProgram: SystemProgram.programId, tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID, rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([buyer2])
        .rpc();

      const tkt     = await program.account.ticketAccount.fetch(ticketKey);
      const listing = await program.account.listingAccount.fetch(listingKey);
      expect(tkt.isListed).to.be.true;
      expect(listing.price.toNumber()).to.equal(PRICE.toNumber());
    });

    it("buyer cancels listing and reclaims token", async () => {
      await program.methods
        .cancelListing()
        .accounts({
          event: eventKey, ticket: ticketKey, listing: listingKey, mint: mintKey,
          escrowAta, sellerAta, seller: buyer2.publicKey,
          systemProgram: SystemProgram.programId, tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID, rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([buyer2])
        .rpc();

      const tkt = await program.account.ticketAccount.fetch(ticketKey);
      expect(tkt.isListed).to.be.false;
    });

    it("buys ticket with correct 5% royalty to organiser", async () => {
      const evCfg = await program.account.eventAccount.fetch(eventKey);
      if (evCfg.royaltyBps !== 500) {
        await program.methods
          .updateEvent({
            name: null,
            description: null,
            venue: null,
            metadataUri: null,
            eventStart: null,
            eventEnd: null,
            resaleAllowed: null,
            maxResalePrice: null,
            royaltyBps: 500,
          })
          .accounts({ event: eventKey, authority: organiser.publicKey })
          .signers([organiser])
          .rpc();
      }

      // Re-list
      await program.methods
        .listTicket(PRICE)
        .accounts({
          event: eventKey, ticket: ticketKey, listing: listingKey, mint: mintKey,
          sellerAta, escrowAta, seller: buyer2.publicKey,
          systemProgram: SystemProgram.programId, tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID, rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([buyer2])
        .rpc();

      const evBefore     = await program.account.eventAccount.fetch(eventKey);
      const royaltyPk    = evBefore.royaltyReceiver as PublicKey;
      expect(evBefore.royaltyBps).to.equal(500);
      const orgBefore    = await conn.getBalance(royaltyPk);
      const sellerBefore = await conn.getBalance(buyer2.publicKey);

      const buySig = await program.methods
        .buyTicket()
        .accounts({
          event: eventKey, ticket: ticketKey, listing: listingKey, mint: mintKey,
          escrowAta, buyerAta, seller: buyer2.publicKey,
          royaltyReceiver: royaltyPk, buyer: buyer1.publicKey,
          systemProgram: SystemProgram.programId, tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID, rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([buyer1])
        .rpc();

      await conn.confirmTransaction(buySig, "confirmed");

      const orgAfter    = await conn.getBalance(royaltyPk);
      const sellerAfter = await conn.getBalance(buyer2.publicKey);

      const royalty = orgAfter - orgBefore;
      const expectedRoyalty = Math.floor(PRICE.toNumber() * 500 / 10000);
      expect(royalty).to.equal(expectedRoyalty);

      const sellerReceived = sellerAfter - sellerBefore;
      // seller receives price - royalty + listing rent back
      expect(sellerReceived).to.be.greaterThan(PRICE.toNumber() - royalty - 10000);

      const tkt = await program.account.ticketAccount.fetch(ticketKey);
      expect(tkt.owner.toBase58()).to.equal(buyer1.publicKey.toBase58());
      expect(tkt.resaleCount).to.equal(1);
      expect(tkt.transferCount).to.equal(2);

      const ev = await program.account.eventAccount.fetch(eventKey);
      expect(ev.totalRoyalties.toNumber() - evBefore.totalRoyalties.toNumber()).to.equal(expectedRoyalty);
    });
  });

  // Cancel event 

  describe("cancel_event", () => {
    it("cancels a fresh event (0 check-ins)", async () => {
      const id = new BN(555);
      const [ek] = eventPda(organiser.publicKey, id, progId);

      await program.methods
        .createEvent(baseEventParams(id))
        .accounts({ event: ek, organizerProfile: null, authority: organiser.publicKey, systemProgram: SystemProgram.programId })
        .signers([organiser])
        .rpc();

      await program.methods
        .cancelEvent()
        .accounts({ event: ek, authority: organiser.publicKey })
        .signers([organiser])
        .rpc();

      const ev = await program.account.eventAccount.fetch(ek);
      expect(ev.isCancelled).to.be.true;
      expect(ev.isActive).to.be.false;
    });

    it("blocks cancel after check-ins", async () => {
      const id = new BN(100); // CI event already has 1 check-in
      const [ek] = eventPda(organiser.publicKey, id, progId);
      const evBefore = await program.account.eventAccount.fetch(ek);
      expect(evBefore.totalCheckedIn.toNumber()).to.be.greaterThan(0);
      try {
        await program.methods
          .cancelEvent()
          .accounts({ event: ek, authority: organiser.publicKey })
          .signers([organiser])
          .rpc();
        expect.fail();
      } catch (e: any) {
        const code = e?.error?.errorCode?.code;
        const logs = (e?.logs ?? []).join(" ");
        expect(code === "CannotCancelAfterCheckIn" || logs.includes("CannotCancelAfterCheckIn")).to.be.true;
      }
    });
  });
});