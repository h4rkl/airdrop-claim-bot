// Here we export some useful types and functions for interacting with the Anchor program.
import { AnchorProvider, Program } from '@coral-xyz/anchor';
import { Cluster, PublicKey } from '@solana/web3.js';
import AirdropIDL from '../target/idl/airdrop.json';
import type { Airdrop } from '../target/types/airdrop';

// Re-export the generated IDL and type
export { Airdrop, AirdropIDL };

// The programId is imported from the program IDL.
export const AIRDROP_PROGRAM_ID = new PublicKey(AirdropIDL.address);

// This is a helper function to get the Airdrop Anchor program.
export function getAirdropProgram(provider: AnchorProvider) {
  return new Program(AirdropIDL as Airdrop, provider);
}

// This is a helper function to get the program ID for the Airdrop program depending on the cluster.
export function getAirdropProgramId(cluster: Cluster) {
  switch (cluster) {
    case 'devnet':
    case 'testnet':
    case 'mainnet-beta':
    default:
      return AIRDROP_PROGRAM_ID;
  }
}
