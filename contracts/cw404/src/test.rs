#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        coins, Addr, BlockInfo, Coin, DepsMut, Env, MessageInfo, Response, Uint128,
    };
    use cw20::TokenInfoResponse;
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::{
        contract::{execute, instantiate},
        msg::UserInfoResponse,
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

        wasms.clone().for_each(|w| println!("{:?}", w.attributes));

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

        wasms.clone().for_each(|w| println!("{:?}", w.attributes));

        assert!(wasms
            .clone()
            .find(|w| w
                .attributes
                .iter()
                .any(|a| a.key == "action" && a.value == "transfer"))
            .is_some());

        // assert max token_id is 5
        wasms
            .clone()
            .map(|w| w.attributes.iter().find(|a| a.key == "token_id"))
            .for_each(|a| println!("{:?}", a));
    }
}
