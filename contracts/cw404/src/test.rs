#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr, Uint128};
    use cw20::TokenInfoResponse;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::{
        contract::{execute, instantiate},
        msg::{TokenPoolResponse, UserInfoResponse},
        query::query,
        ContractError, ExecuteMsg, InstantiateMsg, QueryMsg,
    };

    #[test]
    fn test_instantiate() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    name: "test".to_string(),
                    symbol: "test".to_string(),
                    decimals: 18,
                    total_native_supply: Uint128::new(10),
                    token_id_cap: None,
                    minter: None,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: TokenInfoResponse = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::TokenInfo {})
            .unwrap();

        assert_eq!(
            resp,
            TokenInfoResponse {
                name: "test".to_string(),
                symbol: "test".to_string(),
                decimals: 18,
                total_supply: Uint128::new(10000000000000000000),
            }
        );
    }

    #[test]
    fn test_transfers() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(101, "inj"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let cw404_contract = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    name: "test".to_string(),
                    symbol: "test".to_string(),
                    decimals: 18,
                    total_native_supply: Uint128::new(5),
                    token_id_cap: None,
                    minter: None,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        // whitelist self to prevent burning
        app.execute_contract(
            Addr::unchecked("owner"),
            cw404_contract.clone(),
            &ExecuteMsg::SetWhitelist {
                target: Addr::unchecked("owner").to_string(),
                state: true,
            },
            &[],
        )
        .unwrap();

        let resp = app
            .execute_contract(
                Addr::unchecked("owner"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user").to_string(),
                    amount: Uint128::new(5) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert!(wasms
            .clone()
            .find(|w| w
                .attributes
                .iter()
                .any(|a| a.key == "action" && a.value == "transfer"))
            .is_some());

        assert_eq!(
            wasms
                .clone()
                .filter(|w| w
                    .attributes
                    .iter()
                    .any(|a| a.key == "action" && a.value == "mint"))
                .count(),
            5
        );

        let resp: UserInfoResponse = app
            .wrap()
            .query_wasm_smart(
                cw404_contract.clone(),
                &QueryMsg::UserInfo {
                    address: Addr::unchecked("user").to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            resp,
            UserInfoResponse {
                owned: vec![
                    Uint128::new(1),
                    Uint128::new(2),
                    Uint128::new(3),
                    Uint128::new(4),
                    Uint128::new(5)
                ],
                balances: Uint128::new(5) * Uint128::new(10).pow(18)
            }
        );

        let resp = app
            .execute_contract(
                Addr::unchecked("user"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user3").to_string(),
                    amount: Uint128::new(5) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert!(wasms
            .clone()
            .find(|w| w
                .attributes
                .iter()
                .any(|a| a.key == "action" && a.value == "transfer"))
            .is_some());

        // assert max token_id is 5
        assert_eq!(
            wasms
                .clone()
                .filter_map(|w| w.attributes.iter().find(|a| a.key == "token_id"))
                .map(|a| a.value.parse::<u64>().unwrap())
                .reduce(|a, b| a.max(b))
                .unwrap(),
            5
        );
    }

    #[test]
    fn test_cap_limit() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(101, "inj"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let cw404_contract = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    name: "test".to_string(),
                    symbol: "test".to_string(),
                    decimals: 18,
                    total_native_supply: Uint128::new(5),
                    token_id_cap: Some(Uint128::new(8)),
                    minter: None,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        // whitelist self to prevent burning
        app.execute_contract(
            Addr::unchecked("owner"),
            cw404_contract.clone(),
            &ExecuteMsg::SetWhitelist {
                target: Addr::unchecked("owner").to_string(),
                state: true,
            },
            &[],
        )
        .unwrap();

        // owner -> user transfer
        let resp = app
            .execute_contract(
                Addr::unchecked("owner"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user").to_string(),
                    amount: Uint128::new(5) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        // wasms.clone().for_each(|w| println!("{:?}", w.attributes));

        assert!(wasms
            .clone()
            .find(|w| w
                .attributes
                .iter()
                .any(|a| a.key == "action" && a.value == "transfer"))
            .is_some());

        assert_eq!(
            wasms
                .clone()
                .filter(|w| w
                    .attributes
                    .iter()
                    .any(|a| a.key == "action" && a.value == "mint"))
                .count(),
            5
        );

        let resp: UserInfoResponse = app
            .wrap()
            .query_wasm_smart(
                cw404_contract.clone(),
                &QueryMsg::UserInfo {
                    address: Addr::unchecked("user").to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            resp,
            UserInfoResponse {
                owned: vec![
                    Uint128::new(1),
                    Uint128::new(2),
                    Uint128::new(3),
                    Uint128::new(4),
                    Uint128::new(5)
                ],
                balances: Uint128::new(5) * Uint128::new(10).pow(18)
            }
        );

        // User -> User3 transfer
        let resp = app
            .execute_contract(
                Addr::unchecked("user"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user3").to_string(),
                    amount: Uint128::new(5) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert!(wasms
            .clone()
            .find(|w| w
                .attributes
                .iter()
                .any(|a| a.key == "action" && a.value == "transfer"))
            .is_some());

        assert_eq!(
            wasms
                .clone()
                .filter_map(|w| w.attributes.iter().find(|a| a.key == "token_id"))
                .map(|a| a.value.parse::<u64>().unwrap())
                .reduce(|a, b| a.max(b))
                .unwrap(),
            8
        );

        // User3 -> User2 transfer
        let resp = app
            .execute_contract(
                Addr::unchecked("user3"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user2").to_string(),
                    amount: Uint128::new(5) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert!(wasms
            .clone()
            .find(|w| w
                .attributes
                .iter()
                .any(|a| a.key == "action" && a.value == "transfer"))
            .is_some());

        assert_eq!(
            wasms
                .clone()
                .filter_map(|w| w.attributes.iter().find(|a| a.key == "token_id"))
                .map(|a| a.value.parse::<u64>().unwrap())
                .reduce(|a, b| a.max(b))
                .unwrap(),
            8
        );

        // Increase cap
        app.execute_contract(
            Addr::unchecked("owner"),
            cw404_contract.clone(),
            &ExecuteMsg::SetTokenIdCap {
                cap: Uint128::new(9),
            },
            &[],
        )
        .unwrap();

        //  User2 -> user3 transfer (2 tokens)
        let resp = app
            .execute_contract(
                Addr::unchecked("user2"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user3").to_string(),
                    amount: Uint128::new(2) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert_eq!(
            wasms
                .clone()
                .filter_map(|w| w.attributes.iter().find(|a| a.key == "token_id"))
                .map(|a| a.value.parse::<u64>().unwrap())
                .reduce(|a, b| a.max(b))
                .unwrap(),
            9
        );

        // user3 -> user2 transfer (2 tokens)
        app.execute_contract(
            Addr::unchecked("user3"),
            cw404_contract.clone(),
            &ExecuteMsg::Transfer {
                recipient: Addr::unchecked("user2").to_string(),
                amount: Uint128::new(2) * Uint128::new(10).pow(18),
            },
            &[],
        )
        .unwrap();

        //  User2 -> owner transfer (burn all tokens)
        let resp = app
            .execute_contract(
                Addr::unchecked("user2"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("owner").to_string(),
                    amount: Uint128::new(5) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert!(wasms
            .clone()
            .find(|w| w
                .attributes
                .iter()
                .any(|a| a.key == "action" && a.value == "transfer"))
            .is_some());

        assert_eq!(
            wasms
                .clone()
                .filter_map(|w| w
                    .attributes
                    .iter()
                    .find(|a| a.key == "action" && a.value == "burn"))
                .count(),
            5
        );

        let resp: TokenPoolResponse = app
            .wrap()
            .query_wasm_smart(cw404_contract, &QueryMsg::TokenPool {})
            .unwrap();

        assert_eq!(resp.pool_count, 9);
        assert_eq!(resp.token_id_cap, Uint128::new(9));
    }

    #[test]
    fn test_sequential_mints() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(101, "inj"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let cw404_contract = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    name: "test".to_string(),
                    symbol: "test".to_string(),
                    decimals: 18,
                    total_native_supply: Uint128::new(100),
                    token_id_cap: Some(Uint128::new(100)),
                    minter: None,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();

        // whitelist self to prevent burning
        app.execute_contract(
            Addr::unchecked("owner"),
            cw404_contract.clone(),
            &ExecuteMsg::SetWhitelist {
                target: Addr::unchecked("owner").to_string(),
                state: true,
            },
            &[],
        )
        .unwrap();

        // owner -> user transfer 50 tokens
        let resp = app
            .execute_contract(
                Addr::unchecked("owner"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user").to_string(),
                    amount: Uint128::new(50) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert_eq!(
            wasms
                .clone()
                .filter(|w| w
                    .attributes
                    .iter()
                    .any(|a| a.key == "action" && a.value == "mint"))
                .count(),
            50
        );

        let resp: TokenPoolResponse = app
            .wrap()
            .query_wasm_smart(cw404_contract.clone(), &QueryMsg::TokenPool {})
            .unwrap();

        assert_eq!(resp.pool_count, 0);
        assert_eq!(resp.token_id_cap, Uint128::new(100));

        let resp: UserInfoResponse = app
            .wrap()
            .query_wasm_smart(
                cw404_contract.clone(),
                &QueryMsg::UserInfo {
                    address: Addr::unchecked("user").to_string(),
                },
            )
            .unwrap();

        assert_eq!(
            resp,
            UserInfoResponse {
                owned: (1..=50).map(|i| Uint128::new(i)).collect(),
                balances: Uint128::new(50) * Uint128::new(10).pow(18)
            }
        );

        // User -> User3 transfer 25 tokens (burns 25 tokens, mint 25 tokens)
        let resp = app
            .execute_contract(
                Addr::unchecked("user"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user3").to_string(),
                    amount: Uint128::new(25) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert_eq!(
            wasms
                .clone()
                .filter_map(|w| w.attributes.iter().find(|a| a.key == "token_id"))
                .map(|a| a.value.parse::<u64>().unwrap())
                .reduce(|a, b| a.max(b))
                .unwrap(),
            75
        );

        // User3 -> User2 transfer 25 tokens (burns 25 tokens, mint 25 tokens)
        let resp = app
            .execute_contract(
                Addr::unchecked("user3"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user2").to_string(),
                    amount: Uint128::new(25) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert_eq!(
            wasms
                .clone()
                .filter_map(|w| w.attributes.iter().find(|a| a.key == "token_id"))
                .map(|a| a.value.parse::<u64>().unwrap())
                .reduce(|a, b| a.max(b))
                .unwrap(),
            100
        );
        // cap reached, next mints will reuse token_id lexically
        let resp: TokenPoolResponse = app
            .wrap()
            .query_wasm_smart(cw404_contract.clone(), &QueryMsg::TokenPool {})
            .unwrap();

        assert_eq!(resp.pool_count, 50);
        assert_eq!(resp.token_id_cap, Uint128::new(100));

        //  User2 -> user transfer 25 tokens (burns 25 tokens, remint 25 tokens)
        let resp = app
            .execute_contract(
                Addr::unchecked("user2"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user").to_string(),
                    amount: Uint128::new(25) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        assert_eq!(
            wasms
                .clone()
                .filter_map(|w| w.attributes.iter().find(|a| a.key == "token_id"))
                .map(|a| a.value.parse::<u64>().unwrap())
                .reduce(|a, b| a.max(b))
                .unwrap(),
            100
        );

        let resp: TokenPoolResponse = app
            .wrap()
            .query_wasm_smart(cw404_contract.clone(), &QueryMsg::TokenPool {})
            .unwrap();

        assert_eq!(resp.pool_count, 50);
        assert_eq!(resp.token_id_cap, Uint128::new(100));

        // Mint out all supply
        // owner -> user transfer 50 tokens
        let resp = app
            .execute_contract(
                Addr::unchecked("owner"),
                cw404_contract.clone(),
                &ExecuteMsg::Transfer {
                    recipient: Addr::unchecked("user").to_string(),
                    amount: Uint128::new(50) * Uint128::new(10).pow(18),
                },
                &[],
            )
            .unwrap();

        let wasms = resp.events.iter().filter(|ev| ev.ty == "wasm");

        // wasms.clone().for_each(|w| println!("{:?}", w.attributes));

        assert_eq!(
            wasms
                .clone()
                .filter(|w| w
                    .attributes
                    .iter()
                    .any(|a| a.key == "action" && a.value == "mint"))
                .count(),
            50
        );

        let resp: TokenPoolResponse = app
            .wrap()
            .query_wasm_smart(cw404_contract.clone(), &QueryMsg::TokenPool {})
            .unwrap();

        assert_eq!(resp.pool_count, 0);
        assert_eq!(resp.token_id_cap, Uint128::new(100));
    }
}
