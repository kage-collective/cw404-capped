

```bash
INJ_ADDRESS=inj1635cku3flgssj6q92l48juyw7c5war2v52vj8q

yes 12345678 | injectived tx wasm store ./artifacts/cw404.wasm \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --gas-prices=500000000inj --gas=20000000 \
--node=https://testnet.sentry.tm.injective.network:443


# CODE_ID=7761

CODE_ID=8417 # original cw404
# CODE_ID=8408 # sequential remints

# CW404_INIT='{"name":"LIMIT BREAK","symbol":"LBK2","decimals":18,"total_native_supply":"15000"}'
# CW404_INIT='{"name":"LIMIT BREAK","symbol":"LBK3","decimals":18,"total_native_supply":"1000"}'
CW404_INIT='{"name":"OG404","symbol":"OG4042","decimals":18,"total_native_supply":"1000"}'

yes 12345678 | injectived tx wasm instantiate $CODE_ID $CW404_INIT --label="Init CW404" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://testnet.sentry.tm.injective.network:443 --dry-run

# TEST_404=inj1h4nqsp435wnzaau4sqsuvqkxngkuad9m0dawnx

# TEST_404=inj1ta0jferds2wut90stl0l7sfxpftfjw5yejj96z # LBK2
# TEST_404=inj1eu902t6hpusk0mmt9fk5xu4zwhyufgsc349ae0 # LBK3
# TEST_404=inj1j7xjm9c8v5gju3xesyeqmpgp360plfaeg70nps # OG404
# TEST_404=inj1mdq3x4er0q52cs0sfrexndxuql095mt4fhayjg # OG4042

WHITELIST_SELF='{"set_whitelist":{"target":"'$INJ_ADDRESS'","state":true}}'

yes 12345678 | injectived tx wasm execute $TEST_404 "$WHITELIST_SELF" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --node=https://testnet.sentry.tm.injective.network:443 --dry-run

BALANCE_404='{"balance":{"address": "'$INJ_ADDRESS'"}}'
injectived query wasm contract-state smart $TEST_404 "$BALANCE_404" --node=https://testnet.sentry.tm.injective.network:443 --output json


INJ_USER2=inj15x87psc989d5yeyxs9vpe5aa7wlaak6730v43q

TRANSFER_404='{"transfer":{"recipient":"'$INJ_USER2'","amount":"50000000000000000000"}}'

yes 12345678 | injectived tx wasm execute $TEST_404 "$TRANSFER_404" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=50000000 --node=https://testnet.sentry.tm.injective.network:443 --dry-run


TRANSFER_404='{"transfer":{"recipient":"'$INJ_USER2'","amount":"350000000000000000000"}}'

yes 12345678 | injectived tx wasm execute $TEST_404 "$TRANSFER_404" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=50000000 --node=https://testnet.sentry.tm.injective.network:443 --dry-run


INJ_USER3=inj1y7p4rctk7y6tkjh3zr4hpqu5dyr334m8wfdvr7

TRANSFER_404='{"transfer":{"recipient":"'$INJ_USER2'","amount":"100000000000000000000"}}'

yes 12345678 | injectived tx wasm execute $TEST_404 "$TRANSFER_404" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=50000000 --node=https://testnet.sentry.tm.injective.network:443 --dry-run


INJ_USER4=inj1eukw5lgakapal054nldataq00tv3cuqn2g5405

TRANSFER_404='{"transfer":{"recipient":"'$INJ_USER4'","amount":"100000000000000000000"}}'

yes 12345678 | injectived tx wasm execute $TEST_404 "$TRANSFER_404" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=50000000 --node=https://testnet.sentry.tm.injective.network:443 --dry-run



TOKEN_POOL_QUERY='{"token_pool":{}}'
injectived query wasm contract-state smart $TEST_404 "$TOKEN_POOL_QUERY" --node=https://testnet.sentry.tm.injective.network:443 --output json



SET_CAP='{"set_token_id_cap":{"cap":"1100"}}'

yes 12345678 | injectived tx wasm execute $TEST_404 "$SET_CAP" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=50000000 --node=https://testnet.sentry.tm.injective.network:443 --dry-run
```