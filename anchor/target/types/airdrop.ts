/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/airdrop.json`.
 */
export type Airdrop = {
  "address": "HQ9qykbDvtGPm5LtLzCyn25ntRwi9DePTevwA6o9mXAZ",
  "metadata": {
    "name": "airdrop",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "Created with Anchor"
  },
  "instructions": [
    {
      "name": "claimTokens",
      "discriminator": [
        108,
        216,
        210,
        231,
        0,
        212,
        42,
        64
      ],
      "accounts": [
        {
          "name": "pool",
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "mint"
              },
              {
                "kind": "const",
                "value": [
                  97,
                  105,
                  114,
                  100,
                  114,
                  111,
                  112,
                  95,
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "userTokenAccount",
          "writable": true
        },
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "poolTokenAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "mint"
              },
              {
                "kind": "const",
                "value": [
                  97,
                  105,
                  114,
                  100,
                  114,
                  111,
                  112,
                  95,
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "userClaim",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "const",
                "value": [
                  97,
                  105,
                  114,
                  100,
                  114,
                  111,
                  112,
                  95,
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "mint"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "initializePool",
      "discriminator": [
        95,
        180,
        10,
        172,
        84,
        174,
        232,
        40
      ],
      "accounts": [
        {
          "name": "authority",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "mint"
              },
              {
                "kind": "const",
                "value": [
                  97,
                  105,
                  114,
                  100,
                  114,
                  111,
                  112,
                  95,
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "from",
          "writable": true
        },
        {
          "name": "poolTokenAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "mint"
              },
              {
                "kind": "const",
                "value": [
                  97,
                  105,
                  114,
                  100,
                  114,
                  111,
                  112,
                  95,
                  112,
                  114,
                  111,
                  116,
                  111,
                  99,
                  111,
                  108
                ]
              }
            ]
          }
        },
        {
          "name": "mint"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "airdropPool",
      "discriminator": [
        196,
        25,
        1,
        13,
        71,
        9,
        85,
        148
      ]
    },
    {
      "name": "userClaim",
      "discriminator": [
        228,
        142,
        195,
        181,
        228,
        147,
        32,
        209
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidTokenPoolAccount",
      "msg": "Invalid pool token account."
    },
    {
      "code": 6001,
      "name": "invalidPoolAddress",
      "msg": "Invalid pool address."
    },
    {
      "code": 6002,
      "name": "alreadyClaimed",
      "msg": "User has already claimed their tokens."
    },
    {
      "code": 6003,
      "name": "invalidAmount",
      "msg": "Invalid amount."
    }
  ],
  "types": [
    {
      "name": "airdropPool",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authority",
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "userClaim",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "user",
            "type": "pubkey"
          },
          {
            "name": "hasClaimed",
            "type": "bool"
          }
        ]
      }
    }
  ]
};
