import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  createAccount,
  createMint,
  getAccount,
  mintTo,
} from '@solana/spl-token';
import { Airdrop } from '../target/types/airdrop';

const AIRDROP_PROTOCOL = 'airdrop_protocol';
const CLAIM_ALLOCATION = 'claim_allocation';

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.Airdrop as Program<Airdrop>;

describe('Airdrop security', () => {
  let poolOwner: Keypair;
  let userAccount: Keypair;
  let attacker: Keypair;
  let poolOwnerTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let mint: PublicKey;
  let poolTokenAccount: PublicKey;
  let poolPDA: PublicKey;
  let poolAccounting: PublicKey;
  let claimAllocation: PublicKey;

  const poolAmount = toBaseUnits(600_000);
  const allocationAmount = toBaseUnits(1_000);

  beforeAll(async () => {
    poolOwner = Keypair.generate();
    userAccount = Keypair.generate();
    attacker = Keypair.generate();

    await Promise.all(
      [poolOwner, userAccount, attacker].map(async (keypair) => {
        const signature = await provider.connection.requestAirdrop(
          keypair.publicKey,
          LAMPORTS_PER_SOL
        );
        await provider.connection.confirmTransaction(signature);
      })
    );

    mint = await createMint(
      provider.connection,
      poolOwner,
      poolOwner.publicKey,
      null,
      9
    );
    poolOwnerTokenAccount = await createAccount(
      provider.connection,
      poolOwner,
      mint,
      poolOwner.publicKey
    );
    userTokenAccount = await createAccount(
      provider.connection,
      userAccount,
      mint,
      userAccount.publicKey
    );
    await mintTo(
      provider.connection,
      poolOwner,
      mint,
      poolOwnerTokenAccount,
      poolOwner.publicKey,
      toBaseUnits(1_000_000)
    );

    [poolPDA] = PublicKey.findProgramAddressSync(
      [mint.toBuffer(), Buffer.from(AIRDROP_PROTOCOL)],
      program.programId
    );
    [poolTokenAccount] = PublicKey.findProgramAddressSync(
      [poolPDA.toBuffer(), mint.toBuffer(), Buffer.from(AIRDROP_PROTOCOL)],
      program.programId
    );
    [claimAllocation] = PublicKey.findProgramAddressSync(
      [
        userAccount.publicKey.toBuffer(),
        poolPDA.toBuffer(),
        Buffer.from(CLAIM_ALLOCATION),
      ],
      program.programId
    );
    [poolAccounting] = PublicKey.findProgramAddressSync(
      [poolPDA.toBuffer(), Buffer.from('pool_accounting')],
      program.programId
    );
  });

  it('initializes a pool controlled by the mint authority', async () => {
    await program.methods
      .initializePool(new anchor.BN(poolAmount))
      .accountsStrict({
        authority: poolOwner.publicKey,
        poolAuthority: poolPDA,
        from: poolOwnerTokenAccount,
        poolTokenAccount,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([poolOwner])
      .rpc();

    const pool = await program.account.airdropPool.fetch(poolPDA);
    expect(pool.authority.equals(poolOwner.publicKey)).toBe(true);
    expect((await getAccount(provider.connection, poolTokenAccount)).amount).toBe(
      BigInt(poolAmount)
    );
  });

  it('rejects an unapproved allocator and allocations above the pool balance', async () => {
    await expect(
      program.methods
        .registerClaim(new anchor.BN(allocationAmount))
        .accountsStrict({
          poolAuthority: poolPDA,
          authority: attacker.publicKey,
          poolTokenAccount,
          poolAccounting,
          user: userAccount.publicKey,
          claimAllocation,
          mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([attacker])
        .rpc()
    ).rejects.toThrow();

    await expect(
      program.methods
        .registerClaim(new anchor.BN(poolAmount + 1))
        .accountsStrict({
          poolAuthority: poolPDA,
          authority: poolOwner.publicKey,
          poolTokenAccount,
          poolAccounting,
          user: userAccount.publicKey,
          claimAllocation,
          mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([poolOwner])
        .rpc()
    ).rejects.toThrow();
  });

  it('lets a recipient claim only its administrator-approved amount once', async () => {
    await program.methods
      .registerClaim(new anchor.BN(allocationAmount))
      .accountsStrict({
        poolAuthority: poolPDA,
        authority: poolOwner.publicKey,
        poolTokenAccount,
        poolAccounting,
        user: userAccount.publicKey,
        claimAllocation,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([poolOwner])
      .rpc();

    const allocation = await program.account.claimAllocation.fetch(claimAllocation);
    expect(allocation.amount.toNumber()).toBe(allocationAmount);
    expect(allocation.hasClaimed).toBe(false);

    await program.methods
      .claimTokens()
      .accountsStrict({
        poolAuthority: poolPDA,
        userTokenAccount,
        user: userAccount.publicKey,
        poolTokenAccount,
        poolAccounting,
        claimAllocation,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userAccount])
      .rpc();

    const accounting = await program.account.poolAccounting.fetch(poolAccounting);
    expect(accounting.totalAllocated.toNumber()).toBe(allocationAmount);
    expect(accounting.totalClaimed.toNumber()).toBe(allocationAmount);
    expect((await getAccount(provider.connection, userTokenAccount)).amount).toBe(
      BigInt(allocationAmount)
    );
    expect((await getAccount(provider.connection, poolTokenAccount)).amount).toBe(
      BigInt(poolAmount - allocationAmount)
    );

    await expect(
      program.methods
        .claimTokens()
        .accountsStrict({
          poolAuthority: poolPDA,
          userTokenAccount,
          user: userAccount.publicKey,
          poolTokenAccount,
          poolAccounting,
          claimAllocation,
          mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([userAccount])
        .rpc()
    ).rejects.toThrow();

    await program.methods
      .withdrawUnallocated(new anchor.BN(poolAmount - allocationAmount))
      .accountsStrict({
        poolAuthority: poolPDA,
        authority: poolOwner.publicKey,
        poolTokenAccount,
        poolAccounting,
        destination: poolOwnerTokenAccount,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([poolOwner])
      .rpc();

    expect((await getAccount(provider.connection, poolTokenAccount)).amount).toBe(
      0n
    );
  });
});

function toBaseUnits(amount: number): number {
  return amount * LAMPORTS_PER_SOL;
}
