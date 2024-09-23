import {
    PublicKey,
    Keypair,
    Connection,
    LAMPORTS_PER_SOL,
    Transaction,
    TransactionInstruction,
    SystemProgram,
    // sendAndConfirmRawTransaction,
    sendAndConfirmTransaction,
    // TransactionConfirmationStrategy,
} from "@solana/web3.js";

import {
    // getOrCreateAssociatedTokenAccount,
    getAssociatedTokenAddressSync,
    createAssociatedTokenAccountInstruction,
    createTransferCheckedWithTransferHookInstruction,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    createTransferCheckedInstruction,
} from "@solana/spl-token";


const programId = new PublicKey("Bsygg6pgkUnupUAw1QcofEqUNEhYpkn7rZ3u3SUbDvAq");

// spl-token-2022
const tokenProgramId = new PublicKey("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

// spl-token
// const tokenProgramId = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");


// ---------------------

const connection = new Connection("http://127.0.0.1:8899")

const keypair = Keypair.generate();
const user1 = Keypair.generate();
const user2 = Keypair.generate();



const run = async () => {

    const [stateAccount] = PublicKey.findProgramAddressSync([
        Buffer.from("state")
    ], programId);

    const [tokenAuthority] = PublicKey.findProgramAddressSync([
        Buffer.from("token-authority")
    ], programId);

    const [tokenMint] = PublicKey.findProgramAddressSync([
        Buffer.from("token-mint")
    ], programId);

    const [metaList] = PublicKey.findProgramAddressSync([
        Buffer.from("extra-account-metas"),
        tokenMint.toBuffer(),
    ], programId);



    const latestBlockhash = await connection.getLatestBlockhash();
    const signature = await connection.requestAirdrop(keypair.publicKey, 10 * LAMPORTS_PER_SOL);
    const response = await connection.confirmTransaction({
        signature,
        ...latestBlockhash,
    });


    await init(
        keypair,
        stateAccount,
        tokenMint,
        tokenAuthority,
    );

    await create_meta_list(
        keypair,
        tokenMint,
        metaList,
    );

    const tokenAccounts = await Promise.all([
        new Promise(async (resolve, reject) => {
            let tokenAddress = await createTokenAccount(
                keypair,
                user1,
                tokenMint,
            )

            resolve({
                tokenAddress, user: user1
            })
        }),

        new Promise(async (resolve, reject) => {
            let tokenAddress = await createTokenAccount(
                keypair,
                user2,
                tokenMint,
            )

            resolve({
                tokenAddress, user: user2
            })
        }),
    ]);

    await mintTokens(
        keypair,
        tokenAccounts[0].tokenAddress,
        tokenAuthority,
        tokenMint,
    );

    await transfer_token(
        connection,
        tokenAccounts[0].tokenAddress,
        tokenMint,
        tokenAccounts[1].tokenAddress,
        tokenAccounts[0].user,
        BigInt(1_000_000_000),
        9,
    )

    // await transfer_test(
    //     tokenAccounts[0].tokenAddress,
    //     tokenMint,
    //     tokenAccounts[1].tokenAddress,
    //     tokenAccounts[0].user,
    //     BigInt(1_000_000_000),
    //     9,
    // )


}

const init = async (
    payer,
    stateAccount,
    tokenMint,
    tokenAuthority,
) => {
    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = new TransactionInstruction({
        data: [0, 0, 0, 0, 0, 0, 0, 0, 0],
        keys: [
            {
                isSigner: true,
                isWritable: true,
                pubkey: payer.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: stateAccount,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: TOKEN_2022_PROGRAM_ID,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: tokenMint,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenAuthority,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: SystemProgram.programId,
            },
        ],
        programId,
    })

    const transaction = new Transaction({ ...latestBlockhash });
    transaction.add(instruction);
    transaction.sign(payer);


    let sig = await sendAndConfirmTransaction(connection, transaction, [payer], {
        commitment: "finalized",
    });
    console.log({ name: "init", sig });
}

const create_meta_list = async (
    payer,
    tokenMint,
    metaList,
) => {
    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = new TransactionInstruction({
        data: [0, 0, 0, 0, 0, 0, 0, 0, 4],
        keys: [
            {
                isSigner: true,
                isWritable: true,
                pubkey: payer.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: tokenMint,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: metaList,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: SystemProgram.programId,
            },
        ],
        programId,
    })

    const transaction = new Transaction({ ...latestBlockhash });
    transaction.add(instruction);
    transaction.sign(payer);


    let sig = await sendAndConfirmTransaction(connection, transaction, [payer], {
        commitment: "finalized",
    });
    console.log({ name: "meta list", sig });
}

const createTokenAccount = async (payer, owner, tokenMint) => {
    const latestBlockhash = await connection.getLatestBlockhash();
    const signature = await connection.requestAirdrop(owner.publicKey, LAMPORTS_PER_SOL);
    const response = await connection.confirmTransaction({
        signature,
        ...latestBlockhash,
    });

    const associatedToken = getAssociatedTokenAddressSync(
        tokenMint,
        owner.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID,
    );

    const transaction = new Transaction({ ...latestBlockhash })
        .add(
            createAssociatedTokenAccountInstruction(
                payer.publicKey,
                associatedToken,
                owner.publicKey,
                tokenMint,
                TOKEN_2022_PROGRAM_ID,
                ASSOCIATED_TOKEN_PROGRAM_ID
            )
        );

    let sig = await sendAndConfirmTransaction(connection, transaction, [payer], {
        commitment: "finalized",
    });



    console.log({ name: 'token account', sig, response });

    return associatedToken
}

const mintTokens = async (
    payer,
    receipent,
    tokenAuthority,
    tokenMint,
) => {

    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = new TransactionInstruction({
        data: [0, 0, 0, 0, 0, 0, 0, 0, 1],
        keys: [
            {
                isSigner: true,
                isWritable: true,
                pubkey: payer.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: receipent,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: tokenMint,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenAuthority,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: TOKEN_2022_PROGRAM_ID,
            },
        ],
        programId,
    })

    const transaction = new Transaction({ ...latestBlockhash });
    transaction.add(instruction);

    let sig = await sendAndConfirmTransaction(connection, transaction, [payer], {
        commitment: "finalized",
    });
    console.log({ name: 'mint tokens', sig })

}

const transfer_token = async (
    connection,
    source,
    tokenMint,
    destination,
    owner,
    amount,
    decimal
) => {

    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = await createTransferCheckedWithTransferHookInstruction(
        connection,
        source,
        tokenMint,
        destination,
        owner.publicKey,
        amount,
        decimal,
        [],
        "finalized",// ? finalized | confirmed?
        TOKEN_2022_PROGRAM_ID,
    );

    console.log(instruction)

    const transaction = new Transaction({ ...latestBlockhash })
        .add(instruction);

    let sig = await sendAndConfirmTransaction(connection, transaction, [owner], {
        commitment: "finalized",
        // skipPreflight: true,
    });
    console.log({ name: 'token transfer', sig });

}

const transfer_test = async (
    source,
    tokenMint,
    destination,
    owner,
    amount,
    decimal
) => {
    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = createTransferCheckedInstruction(
        source,
        tokenMint,
        destination,
        owner.publicKey,
        amount,
        decimal,
        [],
        TOKEN_2022_PROGRAM_ID,
    );

    console.log(instruction)

    const transaction = new Transaction({ ...latestBlockhash })
        .add(instruction);

    let sig = await sendAndConfirmTransaction(connection, transaction, [owner], {
        commitment: "finalized",
    });
    console.log({ name: 'token transfer', sig });
}

run()

