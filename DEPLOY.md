

```bash
INJ_ADDRESS=inj1635cku3flgssj6q92l48juyw7c5war2v52vj8q

yes 12345678 | injectived tx wasm store ./artifacts/cw404_fixed-aarch64.wasm \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --gas-prices=500000000inj --gas=20000000 \
--node=https://testnet.sentry.tm.injective.network:443


CODE_ID=7761


CW404_INIT='{"name":"LIMIT BREAK","symbol":"LBK","decimals":18,"total_native_supply":"15000"}'

yes 12345678 | injectived tx wasm instantiate $CODE_ID $CW404_INIT --label="Init CW404" --from=$(echo $INJ_ADDRESS) --chain-id="injective-888" --yes --gas-prices=500000000inj --gas=20000000 --admin=$(echo $INJ_ADDRESS) --node=https://testnet.sentry.tm.injective.network:443
```