# Airdrop Anchor program

## Claim security model

The pool administrator (the mint authority at pool creation) registers one
fixed allocation for each recipient. A recipient can then claim exactly that
allocation once; `claim_tokens` accepts no amount argument.

- `register_claim` is restricted to the stored pool authority and reserves the
  allocation against the PDA token-account balance.
- `claim_tokens` requires the allocated recipient to sign and a token account
  they own for the pool mint.
- `revoke_claim` lets the administrator correct an unclaimed allocation.
- `withdraw_unallocated` can only withdraw the balance not reserved by active
  allocations.

`PoolAccounting` is a separate PDA so existing `AirdropPool` accounts retain
their original serialized layout after an upgrade. Legacy claim PDAs cannot be
used with the new claim instruction.

## Build and test

```sh
anchor build
anchor test
```

The tests cover an unauthorized allocation attempt, over-allocation, a valid
single claim, a duplicate claim, and recovery of unallocated funds.

## Upgrade note

The declared program address is `HQ9qykbDvtGPm5LtLzCyn25ntRwi9DePTevwA6o9mXAZ`.
Upgrade that address only with its existing upgrade-authority wallet; do not
replace it with a newly generated local program keypair. Before upgrading a
live pool, inspect its token balance and prior claim activity because the old
claim path allowed an arbitrary claim amount.
