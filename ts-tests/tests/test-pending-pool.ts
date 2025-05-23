import {expect} from "chai";

import {GAS, GAS_PRICE, GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY} from "./config";
import {createAndFinalizeBlock, customRequest, describeWithFrontier} from "./util";

describeWithFrontier("Frontier RPC (Pending Pool)", (context) => {
    // Solidity: contract test { function multiply(uint a) public pure returns(uint d) {return a * 7;}}
    const TEST_CONTRACT_BYTECODE =
        "0x6080604052348015600f57600080fd5b5060ae8061001e6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b600080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390f35b600060078202905091905056fea265627a7a72315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853a30136fa6c5a7762af7364736f6c63430005110032";

    it("should return a pending transaction", async function () {
        this.timeout(15000);
        const tx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                data: TEST_CONTRACT_BYTECODE,
                value: "0x00",
                gasPrice: GAS_PRICE,
                gas: GAS,
            },
            GENESIS_ACCOUNT_PRIVATE_KEY
        );

        const txHash = (await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])).result;

        const pendingTransaction = (await customRequest(context.web3, "eth_getTransactionByHash", [txHash])).result;
        // pending transactions do not know yet to which block they belong to
        expect(pendingTransaction).to.include({
            blockNumber: null,
            hash: txHash,
            r: "0xc10de43934abede6e3459d3924ecb5f50f8390d1d459b09cfe0bc6d6d27d7016",
            s: "0x3c7b497cfcd394012712edd22135d95785e5a81174c64ed7e2b500a989681aa6",
            v: "0xa99",
        });

        await createAndFinalizeBlock(context.web3);

        const processedTransaction = (await customRequest(context.web3, "eth_getTransactionByHash", [txHash])).result;
        expect(processedTransaction).to.include({
            hash: txHash,
            r: "0xc10de43934abede6e3459d3924ecb5f50f8390d1d459b09cfe0bc6d6d27d7016",
            s: "0x3c7b497cfcd394012712edd22135d95785e5a81174c64ed7e2b500a989681aa6",
            v: "0xa99",
        });
    });
});

describeWithFrontier("Frontier RPC (Pending Transaction Count)", (context) => {
	const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

	it("should return pending transaction count", async function () {
		this.timeout(15000);

		// nonce should be 0
		expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT, "latest")).to.eq(0);

		var nonce = 0;
		let sendTransaction = async () => {
			const tx = await context.web3.eth.accounts.signTransaction(
				{
					from: GENESIS_ACCOUNT,
					to: TEST_ACCOUNT,
					value: "0x200", // Must be higher than ExistentialDeposit
					gasPrice: GAS_PRICE,
					gas: GAS,
					nonce: nonce,
				},
				GENESIS_ACCOUNT_PRIVATE_KEY
			);
			nonce = nonce + 1;
			return (await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])).result;
		};

		{
			const pendingTransactionCount = (
				await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ["pending"])
			).result;
			expect(pendingTransactionCount).to.eq("0x0");
		}

		// block 1 should have 1 transaction
		await sendTransaction();
		{
			const pendingTransactionCount = (
				await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ["pending"])
			).result;
			expect(pendingTransactionCount).to.eq("0x1");
		}

		await createAndFinalizeBlock(context.web3);

		{
			const pendingTransactionCount = (
				await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ["pending"])
			).result;
			expect(pendingTransactionCount).to.eq("0x0");
			const processedTransactionCount = (
				await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", [1])
			).result;
			expect(processedTransactionCount).to.eq("0x1");
		}

		// block 2 should have 5 transactions
		for (var _ of Array(5)) {
			await sendTransaction();
		}

		{
			const pendingTransactionCount = (
				await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ["pending"])
			).result;
			expect(pendingTransactionCount).to.eq("0x5");
		}

		await createAndFinalizeBlock(context.web3);

		{
			const pendingTransactionCount = (
				await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", ["pending"])
			).result;
			expect(pendingTransactionCount).to.eq("0x0");
			const processedTransactionCount = (
				await customRequest(context.web3, "eth_getBlockTransactionCountByNumber", [2])
			).result;
			expect(processedTransactionCount).to.eq("0x5");
		}
	});
});
