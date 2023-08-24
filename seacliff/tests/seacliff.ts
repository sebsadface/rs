import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Seacliff } from "../target/types/seacliff";
import { SystemProgram } from '@solana/web3.js';
import { assert } from 'chai';

describe('seacliff', () => {
   // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Seacliff as Program<Seacliff>;

    it('Creates a new contract', async () => {
        // Pre-conditions
        const contractAccount = anchor.web3.Keypair.generate();
        const proposer = anchor.web3.Keypair.generate();

    

        // Define parameters for create_contract
        const goal = 100_000; // Example value
        const lifespan = 1000; // Example value
        const refund_bonus = 10_000; // Example value

        
    });
});
