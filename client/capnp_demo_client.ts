import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    SystemProgram,
    Transaction,
    TransactionInstruction,
} from "@solana/web3.js";
import {Buffer} from 'buffer';
import * as borsh from '@project-serum/borsh'

const PAYER_KEYPAIR = Keypair.generate();

(async () => {
    const connection = new Connection("http://localhost:8899", "confirmed");
    const programId = new PublicKey(
        "EmcSKVqz9rAwFPWDJ6YxvBLNaFjESNMrJkaREVs7MC72"
    );

    // Airdop to Payer
    const signature = await connection.requestAirdrop(PAYER_KEYPAIR.publicKey, LAMPORTS_PER_SOL * 10);
    await connection.confirmTransaction(signature);

    // Create the account
    const [pda, bump] = await PublicKey.findProgramAddressSync(
        [Buffer.from("customaddress"), PAYER_KEYPAIR.publicKey.toBuffer()],
        programId
    );

    // Calculate the discriminator based on the account struct name, in this case, "person" and it's an "account" type (Look in the program's lib.rs).
    // This is used to look up the "descriptor account" that holds the schema for your primary account
    let disc = await getDiscriminator("person", "account");
    console.log(disc);

    const [pda_descriptor, _] = await PublicKey.findProgramAddressSync(
        [Buffer.from(disc)],
        programId
    );

    console.log(`Account Pubkey: ${pda.toString()}`);
    console.log(`Account Descriptor Pubkey: ${pda_descriptor.toString()}`);

    const instructionSchema = borsh.struct([
        borsh.u8('variant'),
        borsh.u32('bump')
    ])

    const buffer = Buffer.alloc(1000)
    // Create instruction for the canpn initialize instruction (variant 0)
    instructionSchema.encode({variant: 0, bump: bump}, buffer)

    const instructionBuffer = buffer.slice(0, instructionSchema.getSpan(buffer))
    const createPDAIx = new TransactionInstruction({
        programId: programId,
        data: instructionBuffer,
        keys: [
            {
                isSigner: true,
                isWritable: true,
                pubkey: PAYER_KEYPAIR.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: pda,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: pda_descriptor,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: SystemProgram.programId,
            },
        ],
    });

    const transaction = new Transaction();
    transaction.add(createPDAIx);

    const txHash = await connection.sendTransaction(transaction, [PAYER_KEYPAIR]);
    console.log(`Created account and descriptor successfully. Tx Hash: ${txHash}`);
    await connection.confirmTransaction(txHash);

    let pda_account = await connection.getAccountInfo(pda);
    console.log("Account:")
    console.log(JSON.stringify(pda_account));

    console.log("Account Descriptor:")
    let pda_descriptor_account = await connection.getAccountInfo(pda_descriptor);
    console.log(JSON.stringify(pda_descriptor_account));

    console.log("TX 2");
    await sleep(1000);

    //2nd instruction
    const deserializeCapnpDemoInstruction = borsh.struct([
        borsh.u8('variant')
    ])

    const buffer2 = Buffer.alloc(1000)
    // Create the instruction to deserialize the newly created account data on-chain
    deserializeCapnpDemoInstruction.encode({variant: 1}, buffer2)

    const deserializeCapnpDemoInstructionBuffer = buffer2.slice(0, deserializeCapnpDemoInstruction.getSpan(buffer2))
    const createPDAIx2 = new TransactionInstruction({
        programId: programId,
        data: deserializeCapnpDemoInstructionBuffer,
        keys: [
            {
                isSigner: true,
                isWritable: true,
                pubkey: PAYER_KEYPAIR.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: pda,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: pda_descriptor,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: SystemProgram.programId,
            },
        ],
    });

    const deserializeCapnpDemoTx = new Transaction();
    deserializeCapnpDemoTx.add(createPDAIx2);

    const deserializeCapnpDemoTxHash = await connection.sendTransaction(deserializeCapnpDemoTx, [PAYER_KEYPAIR]);
    console.log(`Created PDA successfully. Tx Hash: ${deserializeCapnpDemoTxHash}`);
    await connection.confirmTransaction(deserializeCapnpDemoTxHash);

    let pda_descriptor_account2 = await connection.getAccountInfo(pda_descriptor);
    console.log(JSON.stringify(pda_descriptor_account2));
})();

async function getDiscriminator(account_name: string, account_type: string): Promise<Uint8Array> {
    const response = await fetch("https://test.api.brytelands.io/discriminator-offline/" + account_name + "/" + account_type);
    // @ts-ignore
    return await response.json();
}

export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));