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
  createAssociatedTokenAccountInstruction,
  createMint,
  getAccount,
  getAssociatedTokenAddress,
  mintTo,
} from '@solana/spl-token';
import { Airdrop } from '../target/types/airdrop';

const MINT_AIRDROP_POOL = "mint_airdrop_pool";

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

  // Before all tests, set up accounts and mint tokens
  beforeAll(async () => {
    // Set up the pool account
    poolOwner = Keypair.generate();
    userAccount = Keypair.generate();
    const accounts = await Promise.all(
      [
        { label: 'poolOwner', publicKey: poolOwner.publicKey },
        { label: 'userAccount', publicKey: userAccount.publicKey },
      ].map(async ({ label, publicKey }) =>
        provider.connection
          .confirmTransaction({
            ...(await provider.connection.getLatestBlockhash('confirmed')),
            signature: await provider.connection.requestAirdrop(
              publicKey,
              LAMPORTS_PER_SOL
            ),
          })
          .then(() => label)
      )
    );
    console.log('Airdropped SOL to accounts:', accounts);

    // Create a mint
    mint = await createMint(
      provider.connection,
      poolOwner,
      poolOwner.publicKey,
      null,
      9
    );

    // Create token accounts for the pool and the user
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
      tolamports(1000000)
    );
    const balance = await provider.connection.getTokenAccountBalance(
      poolOwnerTokenAccount
    );
    console.log(
      `Token balance: ${
        balance.value.uiAmountString
      } tokens of ${mint.toString()}`
    );
  });

  // Test initializing the pool
  it('Initializes the airdrop pool', async () => {
    const [poolPDA] = await anchor.web3.PublicKey.findProgramAddress(
      [
        mint.toBuffer(),
        Buffer.from(MINT_AIRDROP_POOL),
      ],
      program.programId
    );
    const poolTokenAccount = await getAssociatedTokenAddress(
      mint,
      poolPDA,
      true
    );
    const createATAIx = createAssociatedTokenAccountInstruction(
      poolOwner.publicKey,
      poolTokenAccount,
      poolPDA,
      mint
    );
    const tx = new anchor.web3.Transaction().add(createATAIx);
    await provider.sendAndConfirm(tx, [poolOwner]);

    await program.methods
      .initializePool(new anchor.BN(tolamports(600000)))
      .accountsStrict({
        authority: poolOwner.publicKey,
        pool: poolPDA,
        from: poolOwnerTokenAccount,
        poolTokenAccount,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([poolOwner])
      .rpc();

    const poolAccountData = await program.account.airdropPool.fetch(poolPDA);
    expect(poolAccountData.authority.equals(poolOwner.publicKey)).toBe(true);

    // Check pool token account balance
    const poolTokenAccountInfo = await getAccount(provider.connection, poolTokenAccount);
    expect(poolTokenAccountInfo.amount).toBe(BigInt(tolamports(600000)));

    // Check pool owner token account balance
    const poolOwnerTokenAccountInfo = await getAccount(provider.connection, poolOwnerTokenAccount);
    expect(poolOwnerTokenAccountInfo.amount).toBe(BigInt(tolamports(400000)));

    // Verify PDA
    const [expectedPoolPDA, _] = await PublicKey.findProgramAddressSync(
      [ mint.toBuffer(), Buffer.from(MINT_AIRDROP_POOL) ],
      program.programId
    );
    expect(poolPDA.equals(expectedPoolPDA)).toBe(true);
  });

  // Test claiming tokens
  // it('Claims tokens from the airdrop pool', async () => {
  //   userAccount = Keypair.generate();
  //   await provider.connection.requestAirdrop(userAccount.publicKey, LAMPORTS_PER_SOL);

  //   const [poolPDA, _] = await PublicKey.findProgramAddressSync(
  //     [Buffer.from("mint_airdrop_pool")],
  //     program.programId
  //   );

  //   const [userClaimPDA, __] = await PublicKey.findProgramAddressSync(
  //     [userAccount.publicKey.toBuffer(), Buffer.from("user_claim")],
  //     program.programId
  //   );

  //   await program.methods
  //     .claimTokens(tolamports(1000))
  //     .accountsStrict({
  //       pool: poolPDA,
  //       userClaim: userClaimPDA,
  //       poolTokenAccount: poolTokenAccount,
  //       userTokenAccount: userTokenAccount,
  //       user: userAccount.publicKey,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       systemProgram: SystemProgram.programId,
  //     })
  //     .signers([userAccount])
  //     .rpc();

  //   const userClaimData = await program.account.userClaim.fetch(userClaimPDA);
  //   expect(userClaimData.hasClaimed).toBe(true);
  // });
});

function tolamports(amount: number): anchor.BN {
  return amount * LAMPORTS_PER_SOL;
}
