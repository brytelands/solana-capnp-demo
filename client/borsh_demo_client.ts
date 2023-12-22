import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    SystemProgram,
    Transaction,
    TransactionInstruction,
} from "@solana/web3.js";
import { Buffer } from 'buffer';
import * as borsh from '@project-serum/borsh'

const PAYER_KEYPAIR = Keypair.generate();

(async () => {
    const connection = new Connection("http://localhost:8899", "confirmed");
    const programId = new PublicKey(
        "52mSsF7psVdBwdDAcfXMbcn2dj3m65Vhbo2urPX8QLVc"
    );

    // Airdop to Payer
    const signature = await connection.requestAirdrop(PAYER_KEYPAIR.publicKey, LAMPORTS_PER_SOL * 10);
    await connection.confirmTransaction(signature);

    const [pda, bump] = await PublicKey.findProgramAddressSync(
        [Buffer.from("customaddress"), PAYER_KEYPAIR.publicKey.toBuffer()],
        programId
    );

    let disc = await getDiscriminator("person", "account");
    console.log(disc);

    const [pda_descriptor, _] = await PublicKey.findProgramAddressSync(
        [Buffer.from(disc)],
        programId
    );

    console.log(`PDA Pubkey: ${pda.toString()}`);

    const instructionSchema = borsh.struct([
        borsh.u8('variant'),
        borsh.u32('bump')
    ])

    const buffer = Buffer.alloc(1000)
    instructionSchema.encode({ variant: 2, bump: bump}, buffer)

    const instructionBuffer = buffer.slice(0, instructionSchema.getSpan(buffer))
//Buffer.from(Uint8Array.of(bump))
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
                isSigner:false,
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
    console.log(`Created PDA successfully. Tx Hash: ${txHash}`);
    await connection.confirmTransaction(txHash);

    let pda_descriptor_account = await connection.getAccountInfo(pda_descriptor);
    console.log(JSON.stringify(pda_descriptor_account));

    console.log("TX 2");
    await sleep(1000);
    //2nd instruction
    const instructionSchema2 = borsh.struct([
        borsh.u8('variant')
    ])

    const buffer2 = Buffer.alloc(1000)
    instructionSchema2.encode({ variant: 3 }, buffer2)

    const instructionBuffer2 = buffer2.slice(0, instructionSchema2.getSpan(buffer2))
//Buffer.from(Uint8Array.of(bump))
    const createPDAIx2 = new TransactionInstruction({
        programId: programId,
        data: instructionBuffer2,
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
                isSigner:false,
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

    const transaction2 = new Transaction();
    transaction2.add(createPDAIx2);

    const txHash2 = await connection.sendTransaction(transaction2, [PAYER_KEYPAIR]);
    console.log(`Created PDA successfully. Tx Hash: ${txHash2}`);
    await connection.confirmTransaction(txHash2);

    let pda_descriptor_account2 = await connection.getAccountInfo(pda_descriptor);
    console.log(JSON.stringify(pda_descriptor_account2));
})();

//TODO set to GC url
async function getDiscriminator(account_name: string, account_type: string): Promise<Uint8Array> {
    const response = await fetch("https://test.api.brytelands.io/discriminator-offline/" + account_name + "/" + account_type);
    // @ts-ignore
    return await response.json();
}

export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));