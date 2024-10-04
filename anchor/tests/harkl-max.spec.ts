import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, createMint, getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import { Airdrop } from '../target/types/airdrop';

// Configure the provider to use the local cluster
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const feePayer = provider.wallet as anchor.Wallet;

// Load the program
const program = anchor.workspace.Airdrop as Program<Airdrop>;

// Define the Jest tests
describe('Airdrop Program', () => {
  let poolAccount: Keypair;
  let poolTokenAccount: PublicKey;
  let mint: PublicKey;
  let userAccount: Keypair;
  let userTokenAccount: PublicKey;

  // Before all tests, set up accounts and mint tokens
  beforeAll(async () => {
    // Create a mint
    mint = await createMintHelper(provider);

    // Create token accounts for the pool and the user
    poolTokenAccount = await createTokenAccount(provider, mint);
    userTokenAccount = await createTokenAccount(provider, mint);

    // Fund the pool with some tokens
    await mintTokens(provider, mint, poolTokenAccount, 1000000);

    // Set up the pool account
    poolAccount = Keypair.generate();
  });

  // Test initializing the pool
  it('Initializes the airdrop pool', async () => {
    const [poolPDA, _] = await PublicKey.findProgramAddressSync(
      [Buffer.from("airdrop_pool")],
      program.programId
    );
  
    await program.methods
      .initializePool(new anchor.BN(500000))
      .accounts({
        pool: poolPDA,
        authority: feePayer.publicKey,
        from: poolTokenAccount,
        poolTokenAccount: poolTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
  
    const poolAccountData = await program.account.airdropPool.fetch(poolPDA);
    expect(poolAccountData.authority.equals(feePayer.publicKey)).toBe(true);
  });

  // Test claiming tokens
  it('Claims tokens from the airdrop pool', async () => {
    userAccount = Keypair.generate();
    await provider.connection.requestAirdrop(userAccount.publicKey, LAMPORTS_PER_SOL);
  
    const [poolPDA, _] = await PublicKey.findProgramAddressSync(
      [Buffer.from("airdrop_pool")],
      program.programId
    );
  
    const [userClaimPDA, __] = await PublicKey.findProgramAddressSync(
      [userAccount.publicKey.toBuffer(), Buffer.from("user_claim")],
      program.programId
    );
  
    await program.methods
      .claimTokens(new anchor.BN(1000))
      .accounts({
        pool: poolPDA,
        userClaim: userClaimPDA,
        poolTokenAccount: poolTokenAccount,
        userTokenAccount: userTokenAccount,
        user: userAccount.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([userAccount])
      .rpc();
  
    const userClaimData = await program.account.userClaim.fetch(userClaimPDA);
    expect(userClaimData.hasClaimed).toBe(true);
  });

  // Helper function to create a mint
  async function createMintHelper(provider: anchor.AnchorProvider): Promise<PublicKey> {
    const mint = Keypair.generate();
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(mint.publicKey, LAMPORTS_PER_SOL)
    );
    await createMint(
      provider.connection,
      provider.wallet,
      provider.wallet.publicKey,
      null,
      9,
      mint
    );
    return mint.publicKey;
  }

  // Helper function to create a token account
  async function createTokenAccount(provider: anchor.AnchorProvider, mint: PublicKey): Promise<PublicKey> {
    const tokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      provider.wallet.publicKey
    );
    return tokenAccount.address;
  }

  // Helper function to mint tokens
  async function mintTokens(provider: anchor.AnchorProvider, mint: PublicKey, destination: PublicKey, amount: number): Promise<void> {
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mint,
      destination,
      provider.wallet.publicKey,
      amount
    );
  }
});