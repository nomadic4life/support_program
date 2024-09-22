import {
    PublicKey,
    Keypair,
    Connection,
    LAMPORTS_PER_SOL,
    Transaction,
    TransactionInstruction,
    SystemProgram,
    sendAndConfirmRawTransaction,
    // TransactionConfirmationStrategy,
} from "@solana/web3.js"
// const web3 = require("@solana/web3.js")

console.log("sanity check")
// console.log(web3)

const programId = new PublicKey("Bsygg6pgkUnupUAw1QcofEqUNEhYpkn7rZ3u3SUbDvAq");

// spl-token-2022
const tokenProgramId = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

// spl-token
// const tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");


const keypair = Keypair.generate();

const connection = new Connection("http://127.0.0.1:8899")

console.log(programId, tokenProgramId)

const run = async () => {

    const latestBlockhash = await connection.getLatestBlockhash();
    const signature = await connection.requestAirdrop(keypair.publicKey, LAMPORTS_PER_SOL);
    const response = await connection.confirmTransaction({
        signature,
        ...latestBlockhash,
    });

    console.log({ response })

    const [stateAccount] = PublicKey.findProgramAddressSync([
        Buffer.from("state")
    ], programId)

    const [tokenAutority] = PublicKey.findProgramAddressSync([
        Buffer.from("token-authority")
    ], programId)

    const [tokenMint] = PublicKey.findProgramAddressSync([
        Buffer.from("token-mint")
    ], programId)

    const transaction = new Transaction({ ...latestBlockhash });

    const instruction = new TransactionInstruction({
        data: [0],
        keys: [
            {
                isSigner: true,
                isWritable: true,
                pubkey: keypair.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: stateAccount,
            },

            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenProgramId,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: tokenMint,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenAutority,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: SystemProgram.programId,
            },
        ],
        programId,
    });

    transaction.add(instruction);
    transaction.sign(keypair);

    console.log("SIGNER: ", keypair, keypair.publicKey, keypair.publicKey.toString());
    console.log(transaction);


    const sig = await connection.sendTransaction(transaction, [keypair]);
    // const sig = await sendAndConfirmRawTransaction(connection, transaction, [keypair]);
    console.log({ sig })
}


run()

