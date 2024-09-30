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
    getAssociatedTokenAddressSync,
    createAssociatedTokenAccountInstruction,
    createTransferCheckedWithTransferHookInstruction,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    createTransferCheckedInstruction,
    createInitializeMint2Instruction,
    createMint,
    TOKEN_PROGRAM_ID,
    getOrCreateAssociatedTokenAccount,
    mintTo,
    createAccount,
    createInitializeAccount3Instruction,
} from "@solana/spl-token";

const programId = new PublicKey("CFJr1PdpkQTkdET2utARRc5kJnQgkKyMEdGHbUg85VtS");
const tokenHookProgramId = new PublicKey("ZxyJ96Yj2bYDSj2GdWF696TyJ7L4rk1VSz3mkAwwZAR");

// ---------------------

const connection = new Connection("http://127.0.0.1:8899")

const keypair = Keypair.generate();
const user1 = Keypair.generate();
const user2 = Keypair.generate();



const run = async () => {

    const [stateAccount] = PublicKey.findProgramAddressSync([
        Buffer.from("state")
    ], programId);

    const [fundEscrow] = PublicKey.findProgramAddressSync([
        Buffer.from("fund-escrow")
    ], programId);

    const [poolEscrow] = PublicKey.findProgramAddressSync([
        Buffer.from("pool-escrow")
    ], programId);

    const [tokenAuthority] = PublicKey.findProgramAddressSync([
        Buffer.from("token-authority")
    ], programId);

    const [tokenMint] = PublicKey.findProgramAddressSync([
        Buffer.from("token-mint")
    ], tokenHookProgramId);

    const [metaList] = PublicKey.findProgramAddressSync([
        Buffer.from("extra-account-metas"),
        tokenMint.toBuffer(),
    ], tokenHookProgramId);



    const latestBlockhash = await connection.getLatestBlockhash();
    const signature = await connection.requestAirdrop(keypair.publicKey, 10 * LAMPORTS_PER_SOL);
    const response = await connection.confirmTransaction({
        signature,
        ...latestBlockhash,
    });

    // create usdc token mint
    const usdcTokenMint = await createMint(
        connection,
        keypair,
        keypair.publicKey,
        keypair.publicKey,
        6,
        Keypair.generate(),
        {
            commitment: "finalized"
        },
        TOKEN_PROGRAM_ID,
    );

    await init(
        keypair,
        tokenMint,
        tokenAuthority,
    );

    await createMetaList(
        keypair,
        tokenMint,
        metaList,
    );

    await initProgram(
        keypair,
        stateAccount,
        fundEscrow,
        poolEscrow,

        tokenAuthority,
        tokenMint,
        usdcTokenMint,
    );

    console.log("HERE A")

    const tokenAccounts = await Promise.all([
        new Promise(async (resolve, reject) => {

            try {
                let usdcTokenAddress = await getOrCreateAssociatedTokenAccount(
                    connection,
                    keypair,
                    usdcTokenMint,
                    user1.publicKey,
                    false,
                    "finalized",
                    {
                        commitment: "finalized"
                    },
                    TOKEN_PROGRAM_ID,
                    ASSOCIATED_TOKEN_PROGRAM_ID,
                );

                let tokenAddress = await createTokenAccount(
                    keypair,
                    user1,
                    tokenMint,
                )

                await mintTo(
                    connection,
                    keypair,
                    usdcTokenMint,
                    usdcTokenAddress.address,
                    keypair,
                    10_000_000,
                    [],
                    {
                        commitment: "finalized"
                    },
                    TOKEN_PROGRAM_ID,
                );

                resolve({
                    usdcTokenAddress: usdcTokenAddress.address, tokenAddress, user: user1
                })

            } catch (err) {
                console.log(err)
                reject()
            }
        }),

        new Promise(async (resolve, reject) => {

            try {
                let usdcTokenAddress = await getOrCreateAssociatedTokenAccount(
                    connection,
                    keypair,
                    usdcTokenMint,
                    user2.publicKey,
                    false,
                    "finalized",
                    {
                        commitment: "finalized"
                    },
                    TOKEN_PROGRAM_ID,
                    ASSOCIATED_TOKEN_PROGRAM_ID,
                );

                let tokenAddress = await createTokenAccount(
                    keypair,
                    user2,
                    tokenMint,
                )

                await mintTo(
                    connection,
                    keypair,
                    usdcTokenMint,
                    usdcTokenAddress.address,
                    keypair,
                    10_000_000,
                    [],
                    {
                        commitment: "finalized"
                    },
                    TOKEN_PROGRAM_ID,
                );

                resolve({
                    usdcTokenAddress: usdcTokenAddress.address, tokenAddress, user: user2
                })
            } catch (err) {
                console.log(err)
                reject()
            }
        }),
    ]);

    console.log("HERE B")

    // await createEscrows(
    //     keypair,
    //     tokenAuthority,
    //     fundEscrow,
    //     poolEscrow,

    //     usdcTokenMint,
    //     tokenMint,
    // );


    await claim(
        tokenAccounts[0].user,
        tokenAccounts[0].usdcTokenAddress,
        tokenAccounts[0].tokenAddress,

        stateAccount,

        fundEscrow,
        poolEscrow,

        tokenAuthority,
        tokenMint,
        usdcTokenMint,
    );


    // await mintTokens(
    //     keypair,
    //     tokenAccounts[0].tokenAddress,
    //     tokenAuthority,
    //     tokenMint,
    // );

    // await transfer_token(
    //     connection,
    //     tokenAccounts[0].tokenAddress,
    //     tokenMint,
    //     tokenAccounts[1].tokenAddress,
    //     tokenAccounts[0].user,
    //     BigInt(1_000_000_000),
    //     9,
    // );

    // await program_transfer(
    //     tokenAccounts[0].user,
    //     tokenAccounts[0].tokenAddress,
    //     tokenAccounts[1].tokenAddress,
    //     tokenMint,
    //     programId,
    //     metaList,
    // );

}

const initProgram = async (
    payer,
    stateAccount,
    fundEscrow,
    poolEscrow,

    authority,
    tokenMint,
    usdcTokenMint,
) => {
    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = new TransactionInstruction({
        data: [0],
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
                isWritable: true,
                pubkey: fundEscrow,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: poolEscrow,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: usdcTokenMint,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenMint,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: TOKEN_PROGRAM_ID,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: TOKEN_2022_PROGRAM_ID,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: authority,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: SystemProgram.programId,
            },
        ],
        programId: programId,
    })

    const transaction = new Transaction({ ...latestBlockhash });
    transaction.add(instruction);
    // transaction.sign(payer);

    console.log(transaction)
    console.log(transaction.instructions)
    console.log(transaction.instructions.keys)




    let sig = await sendAndConfirmTransaction(connection, transaction, [payer], {
        commitment: "finalized",
    });
    console.log({ name: "init program", sig });
}

const claim = async (
    signer,
    source,
    receiver,
    stateAccount,

    fundingEscrow,
    poolEscrow,

    tokenAuthority,
    tokenMint,
    usdcTokenMint,
) => {
    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = new TransactionInstruction({
        data: [1],
        keys: [
            {
                isSigner: true,
                isWritable: true,
                pubkey: signer.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: source,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: receiver,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: stateAccount,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: fundingEscrow,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: poolEscrow,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenAuthority,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenMint,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: usdcTokenMint,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: TOKEN_2022_PROGRAM_ID,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: TOKEN_PROGRAM_ID,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenHookProgramId,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: SystemProgram.programId,
            },
        ],
        programId: programId,
    })

    const transaction = new Transaction({ ...latestBlockhash });
    transaction.add(instruction);
    transaction.sign(signer);


    let sig = await sendAndConfirmTransaction(connection, transaction, [signer], {
        commitment: "finalized",
    });
    console.log({ name: "claim -> mint", sig });
}

const init = async (
    payer,
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
        programId: tokenHookProgramId,
    })

    const transaction = new Transaction({ ...latestBlockhash });
    transaction.add(instruction);
    transaction.sign(payer);


    let sig = await sendAndConfirmTransaction(connection, transaction, [payer], {
        commitment: "finalized",
    });
    console.log({ name: "init mint", sig });
}

const createMetaList = async (
    payer,
    tokenMint,
    metaList,
) => {
    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = new TransactionInstruction({
        data: [0, 0, 0, 0, 0, 0, 0, 0, 3],
        keys: [
            {
                isSigner: true,
                isWritable: true,
                pubkey: payer.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: metaList,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: tokenMint,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: SystemProgram.programId,
            },
        ],
        programId: tokenHookProgramId,
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
                ASSOCIATED_TOKEN_PROGRAM_ID,
            )
        );

    let sig = await sendAndConfirmTransaction(connection, transaction, [payer], {
        commitment: "finalized",
    });


    console.log({ name: 'token account', sig, response });

    return associatedToken
}

const createEscrows = async (
    payer,
    authority,
    fundEscrow,
    poolEscrow,

    usdcTokenMint,
    tokenMint,
) => {

    const latestBlockhash = await connection.getLatestBlockhash();
    const transaction = new Transaction({ ...latestBlockhash });
    transaction.add(
        createInitializeAccount3Instruction(
            fundEscrow,
            usdcTokenMint,
            authority,
            // payer.publicKey,
            TOKEN_PROGRAM_ID,
        ),

        createInitializeAccount3Instruction(
            poolEscrow,
            tokenMint,
            authority,
            TOKEN_2022_PROGRAM_ID,
        )
    );

    console.log(transaction.instructions.keys)

    let sig = await sendAndConfirmTransaction(connection, transaction, [payer], {
        commitment: "finalized",
    });
    console.log({ name: 'create escrows', sig })
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
            // {
            //     isSigner: false,
            //     isWritable: false,
            //     pubkey: TOKEN_2022_PROGRAM_ID,
            // },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenProgramId,
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
        // TOKEN_2022_PROGRAM_ID,
        tokenProgramId
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
        // TOKEN_2022_PROGRAM_ID,
        tokenProgramId
    );

    console.log(instruction)

    const transaction = new Transaction({ ...latestBlockhash })
        .add(instruction);

    let sig = await sendAndConfirmTransaction(connection, transaction, [owner], {
        commitment: "finalized",
    });
    console.log({ name: 'token transfer', sig });
}

const program_transfer = async (
    authority,
    source,
    destination,
    tokenMint,
    hookProgram,
    metaList,
) => {

    console.log("HOOK", hookProgram);
    console.log("META", metaList);


    const latestBlockhash = await connection.getLatestBlockhash();
    const instruction = new TransactionInstruction({
        data: [0, 0, 0, 0, 0, 0, 0, 0, 3],
        keys: [
            {
                isSigner: true,
                isWritable: false,
                pubkey: authority.publicKey,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: source,
            },
            {
                isSigner: false,
                isWritable: true,
                pubkey: destination,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenMint,
            },
            // {
            //     isSigner: false,
            //     isWritable: false,
            //     pubkey: TOKEN_2022_PROGRAM_ID,
            // },
            {
                isSigner: false,
                isWritable: false,
                pubkey: tokenProgramId,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: hookProgram,
            },
            {
                isSigner: false,
                isWritable: false,
                pubkey: metaList,
            },

        ],
        programId,
    })

    const transaction = new Transaction({ ...latestBlockhash });
    transaction.add(instruction);

    let sig = await sendAndConfirmTransaction(connection, transaction, [authority], {
        commitment: "finalized",
    });
    console.log({ name: 'mint tokens', sig })
}

const createProgramMint = async () => {

    let instruction = createInitializeMint2Instruction(
        Keypair.generate().publicKey,
        9,
        Keypair.generate().publicKey,
        // Keypair.generate().publicKey,
        TOKEN_2022_PROGRAM_ID,
    );

    console.log(instruction)
    console.log(instruction.data.length)

}

run()
