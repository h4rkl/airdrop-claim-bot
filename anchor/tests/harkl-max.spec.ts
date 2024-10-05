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
  getAssociatedTokenAddress,
  getMint,
  mintTo,
} from '@solana/spl-token';
import { Airdrop } from '../target/types/airdrop';

const AIRDROP_PROTOCOL = 'airdrop_protocol';

// Configure the provider to use the local cluster
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

// Load the program
const program = anchor.workspace.Airdrop as Program<Airdrop>;

// Define the Jest tests
describe('Airdrop Program', () => {
  let poolOwner: Keypair;
  let poolOwnerTokenAccount: PublicKey;
  let mint: PublicKey;
  let userAccount: Keypair;
  let userTokenAccount: PublicKey;
  let userClaim: PublicKey;
  let poolTokenAccount: PublicKey;
  let poolPDA: PublicKey;

  // Before all tests, set up accounts and mint tokens
  beforeAll(async () => {
    poolOwner = Keypair.generate();
    userAccount = Keypair.generate();

    // Airdrop SOL to pool owner and user
    await Promise.all(
      [poolOwner, userAccount].map(async (keypair) => {
        await provider.connection.confirmTransaction(
          await provider.connection.requestAirdrop(
            keypair.publicKey,
            LAMPORTS_PER_SOL
          )
        );
      })
    );

    // Create a mint
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

    // Fund the pool with some tokens
    await mintTo(
      provider.connection,
      poolOwner,
      mint,
      poolOwnerTokenAccount,
      poolOwner.publicKey,
      toLamports(1000000)
    );

    // Calculate the PDA for the pool
    [poolPDA] = PublicKey.findProgramAddressSync(
      [mint.toBuffer(), Buffer.from(AIRDROP_PROTOCOL)],
      program.programId
    );

    // Get the associated token address for the pool PDA
    [poolTokenAccount] = PublicKey.findProgramAddressSync(
      [poolPDA.toBuffer(), mint.toBuffer(), Buffer.from(AIRDROP_PROTOCOL)],
      program.programId
    );

    [userClaim] = PublicKey.findProgramAddressSync(
      [
        userAccount.publicKey.toBuffer(),
        poolPDA.toBuffer(),
        Buffer.from(AIRDROP_PROTOCOL),
      ],
      program.programId
    );
  });

  // Test initializing the pool
  it('Initializes the airdrop pool', async () => {
    await program.methods
      .initializePool(new anchor.BN(toLamports(600000)))
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

    // Fetch the pool account data and check that the authority is set correctly
    const poolAccountData = await program.account.airdropPool.fetch(poolPDA);
    expect(poolAccountData.authority.equals(poolOwner.publicKey)).toBe(true);

    // Check pool token account balance
    const poolTokenAccountInfo = await getAccount(
      provider.connection,
      poolTokenAccount
    );
    expect(poolTokenAccountInfo.amount).toBe(BigInt(toLamports(600000)));

    // Check pool owner token account balance
    const poolOwnerTokenAccountInfo = await getAccount(
      provider.connection,
      poolOwnerTokenAccount
    );
    expect(poolOwnerTokenAccountInfo.amount).toBe(BigInt(toLamports(400000)));
  });

  // Test claiming tokens
  it('Claims tokens from the airdrop pool', async () => {
    // Perform the claim
    await program.methods
      .claimTokens(new anchor.BN(toLamports(1000)))
      .accountsStrict({
        poolAuthority: poolPDA,
        userTokenAccount,
        user: userAccount.publicKey,
        poolTokenAccount,
        userClaim,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([userAccount])
      .rpc();

    const poolTokenAccountInfo = await getAccount(
      provider.connection,
      poolTokenAccount
    );
    const userTokenAccountInfo = await getAccount(
      provider.connection,
      userTokenAccount
    );
    const userClaimData = await program.account.userClaim.fetch(userClaim);

    expect(userClaimData.hasClaimed).toBe(true);
    expect(userTokenAccountInfo.amount).toBe(BigInt(toLamports(1000)));
    expect(poolTokenAccountInfo.amount).toBe(BigInt(toLamports(599000)));
  });
});

function toLamports(amount: number): number {
  return amount * LAMPORTS_PER_SOL;
}
