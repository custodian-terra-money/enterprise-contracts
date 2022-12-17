use crate::contract::{
    instantiate, query_asset_whitelist, query_dao_info, query_nft_whitelist,
    query_total_staked_amount, DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
};
use crate::tests::helpers::{
    assert_member_voting_power, existing_nft_dao_membership, existing_token_dao_membership,
    instantiate_stub_dao, stub_dao_gov_config, stub_dao_membership_info, stub_dao_metadata,
    stub_enterprise_factory_contract, stub_multisig_dao_membership_info,
    stub_nft_dao_membership_info, stub_token_dao_membership_info, stub_token_info, CW20_ADDR,
    NFT_ADDR,
};
use crate::tests::querier::mock_querier::mock_dependencies;
use common::cw::testing::{mock_env, mock_info, mock_query_ctx};
use cosmwasm_std::{wasm_instantiate, Addr, Decimal, SubMsg, Timestamp, Uint128, Uint64};
use cw20::{Cw20Coin, MinterResponse};
use cw20_base::msg::InstantiateMarketingInfo;
use cw_asset::AssetInfo;
use cw_utils::Duration;
use enterprise_protocol::api::DaoMembershipInfo::{Existing, New};
use enterprise_protocol::api::DaoType::{Multisig, Nft, Token};
use enterprise_protocol::api::NewMembershipInfo::{NewMultisig, NewNft, NewToken};
use enterprise_protocol::api::ProposalActionType::{UpdateAssetWhitelist, UpdateNftWhitelist};
use enterprise_protocol::api::{
    DaoCouncil, DaoGovConfig, DaoMetadata, DaoSocialData, ExistingDaoMembershipMsg, Logo,
    MultisigMember, NewDaoMembershipMsg, NewMultisigMembershipInfo, NewNftMembershipInfo,
    NewTokenMembershipInfo, TokenMarketingInfo,
};
use enterprise_protocol::error::DaoError::{
    InvalidExistingMultisigContract, InvalidExistingNftContract, InvalidExistingTokenContract,
};
use enterprise_protocol::error::{DaoError, DaoResult};
use enterprise_protocol::msg::InstantiateMsg;
use DaoError::{InvalidArgument, ZeroInitialWeightMember};

const CW20_CODE_ID: u64 = 5;
const CW721_CODE_ID: u64 = 6;
const CW3_FIXED_MULTISIG_CODE_ID: u64 = 6;

const TOKEN_NAME: &str = "some_token";
const TOKEN_SYMBOL: &str = "SMBL";
const TOKEN_DECIMALS: u8 = 6;
const TOKEN_MARKETING_OWNER: &str = "marketing_owner";
const TOKEN_LOGO_URL: &str = "logo_url";
const TOKEN_PROJECT_NAME: &str = "project_name";
const TOKEN_PROJECT_DESCRIPTION: &str = "project_description";

const NFT_NAME: &str = "some_nft";
const NFT_SYMBOL: &str = "NFTSM";

const MINTER: &str = "minter";

#[test]
fn instantiate_stores_dao_metadata() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    let info = mock_info("sender", &[]);

    let current_time = Timestamp::from_seconds(1317);
    env.block.time = current_time;

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    let metadata = DaoMetadata {
        name: "Dao name".to_string(),
        description: Some("Dao description".to_string()),
        logo: Logo::Url("logo_url".to_string()),
        socials: DaoSocialData {
            github_username: Some("github".to_string()),
            discord_username: Some("discord".to_string()),
            twitter_username: Some("twitter".to_string()),
            telegram_username: Some("telegram".to_string()),
        },
    };
    let dao_gov_config = DaoGovConfig {
        quorum: Decimal::from_ratio(13u8, 23u8),
        threshold: Decimal::from_ratio(7u8, 29u8),
        vote_duration: 65,
        unlocking_period: Duration::Height(113),
        minimum_deposit: Some(17u8.into()),
    };
    let dao_council = Some(DaoCouncil {
        members: vec!["council_member1".to_string(), "council_member2".to_string()],
        allowed_proposal_action_types: Some(vec![UpdateAssetWhitelist, UpdateNftWhitelist]),
    });
    let asset_whitelist = vec![
        AssetInfo::native("luna"),
        AssetInfo::cw20(Addr::unchecked(CW20_ADDR)),
    ];
    let nft_whitelist = vec![Addr::unchecked("nft_addr1"), Addr::unchecked("nft_addr2")];
    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: metadata.clone(),
            dao_gov_config: dao_gov_config.clone(),
            dao_council: dao_council.clone(),
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: "enterprise_factory_addr".to_string(),
            asset_whitelist: Some(asset_whitelist.clone()),
            nft_whitelist: Some(nft_whitelist.clone()),
        },
    )?;

    let dao_info = query_dao_info(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(dao_info.creation_date, current_time);
    assert_eq!(dao_info.metadata, metadata);
    assert_eq!(dao_info.dao_code_version, Uint64::from(2u8));
    assert_eq!(dao_info.dao_type, Token);
    assert_eq!(dao_info.gov_config, dao_gov_config);
    assert_eq!(dao_info.dao_council, dao_council);
    assert_eq!(
        dao_info.enterprise_factory_contract,
        Addr::unchecked("enterprise_factory_addr")
    );

    let asset_whitelist_response = query_asset_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(asset_whitelist_response.assets, asset_whitelist);

    let nft_whitelist_response = query_nft_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(nft_whitelist_response.nfts, nft_whitelist);

    let total_staked = query_total_staked_amount(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(total_staked.total_staked_amount, Uint128::zero());

    Ok(())
}

#[test]
fn instantiate_existing_token_membership_stores_proper_info() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_token_dao_membership(CW20_ADDR),
        None,
    )?;

    let dao_info = query_dao_info(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(dao_info.dao_type, Token);
    assert_eq!(dao_info.dao_membership_contract, Addr::unchecked(CW20_ADDR));

    Ok(())
}

#[test]
fn instantiate_existing_nft_membership_stores_proper_info() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier.with_num_tokens(&[(NFT_ADDR, 1000u64)]);

    instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        existing_nft_dao_membership(NFT_ADDR),
        None,
    )?;

    let dao_info = query_dao_info(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(dao_info.dao_type, Nft);
    assert_eq!(dao_info.dao_membership_contract, Addr::unchecked(NFT_ADDR));

    Ok(())
}

// TODO: probably has to be replaced by an integration test and deleted
#[ignore]
#[test]
fn instantiate_existing_multisig_membership_stores_proper_info() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_multisig_members(&[("random_cw3_multisig", &[("sender", 10u64)])]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: None,
            dao_membership_info: Existing(ExistingDaoMembershipMsg {
                dao_type: Multisig,
                membership_contract_addr: "random_cw3_multisig".to_string(),
            }),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    )?;

    let dao_info = query_dao_info(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(dao_info.dao_type, Multisig);
    assert_eq!(
        dao_info.dao_membership_contract,
        Addr::unchecked("random_cw3_multisig")
    );

    Ok(())
}

#[test]
fn instantiate_existing_token_membership_with_not_valid_cw20_contract_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let dao_metadata = stub_dao_metadata();
    let dao_gov_config = stub_dao_gov_config();

    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata,
            dao_gov_config,
            dao_council: None,
            dao_membership_info: stub_dao_membership_info(Token, "non_cw20_addr"),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(result, Err(InvalidExistingTokenContract));

    Ok(())
}

#[test]
fn instantiate_existing_nft_membership_with_not_valid_cw721_contract_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let result = instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        stub_dao_membership_info(Nft, "non_cw721_addr"),
        None,
    );

    assert_eq!(result, Err(InvalidExistingNftContract),);

    Ok(())
}

#[test]
fn instantiate_existing_multisig_membership_with_not_valid_cw3_contract_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let result = instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        stub_dao_membership_info(Multisig, "non_cw3_addr"),
        None,
    );

    assert_eq!(result, Err(InvalidExistingMultisigContract));

    Ok(())
}

#[test]
fn instantiate_new_token_membership_instantiates_new_cw20_contract() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let membership_info = NewToken(NewTokenMembershipInfo {
        token_name: TOKEN_NAME.to_string(),
        token_symbol: TOKEN_SYMBOL.to_string(),
        token_decimals: TOKEN_DECIMALS,
        initial_token_balances: vec![Cw20Coin {
            address: "my_address".to_string(),
            amount: 1234u128.into(),
        }],
        token_mint: Some(MinterResponse {
            minter: MINTER.to_string(),
            cap: Some(123456789u128.into()),
        }),
        token_marketing: Some(TokenMarketingInfo {
            project: Some(TOKEN_PROJECT_NAME.to_string()),
            description: Some(TOKEN_PROJECT_DESCRIPTION.to_string()),
            marketing_owner: Some(TOKEN_MARKETING_OWNER.to_string()),
            logo_url: Some(TOKEN_LOGO_URL.to_string()),
        }),
    });
    let asset_whitelist = vec![
        AssetInfo::native("luna"),
        AssetInfo::cw20(Addr::unchecked("allowed_token")),
    ];
    let nft_whitelist = vec![Addr::unchecked("nft_addr1"), Addr::unchecked("nft_addr2")];
    let response = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: None,
            dao_membership_info: New(NewDaoMembershipMsg {
                membership_contract_code_id: CW20_CODE_ID,
                membership_info,
            }),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: Some(asset_whitelist.clone()),
            nft_whitelist: Some(nft_whitelist.clone()),
        },
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::reply_on_success(
            wasm_instantiate(
                CW20_CODE_ID,
                &cw20_base::msg::InstantiateMsg {
                    name: TOKEN_NAME.to_string(),
                    symbol: TOKEN_SYMBOL.to_string(),
                    decimals: TOKEN_DECIMALS,
                    initial_balances: vec![Cw20Coin {
                        address: "my_address".to_string(),
                        amount: 1234u128.into()
                    },],
                    mint: Some(MinterResponse {
                        minter: MINTER.to_string(),
                        cap: Some(123456789u128.into())
                    }),
                    marketing: Some(InstantiateMarketingInfo {
                        project: Some(TOKEN_PROJECT_NAME.to_string()),
                        description: Some(TOKEN_PROJECT_DESCRIPTION.to_string()),
                        marketing: Some(TOKEN_MARKETING_OWNER.to_string()),
                        logo: Some(cw20::Logo::Url(TOKEN_LOGO_URL.to_string())),
                    }),
                },
                vec![],
                TOKEN_NAME.to_string(),
            )?,
            DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
        )]
    );

    let asset_whitelist_response = query_asset_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(asset_whitelist_response.assets, asset_whitelist);

    let nft_whitelist_response = query_nft_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(nft_whitelist_response.nfts, nft_whitelist);

    // TODO: confirm that after reply() is called, the DAO membership contract is properly stored

    Ok(())
}

#[test]
fn instantiate_new_token_membership_with_zero_initial_balance_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let membership_info = NewToken(NewTokenMembershipInfo {
        token_name: TOKEN_NAME.to_string(),
        token_symbol: TOKEN_SYMBOL.to_string(),
        token_decimals: TOKEN_DECIMALS,
        initial_token_balances: vec![
            Cw20Coin {
                address: "my_address".to_string(),
                amount: 1234u128.into(),
            },
            Cw20Coin {
                address: "another_address".to_string(),
                amount: Uint128::zero(),
            },
        ],
        token_mint: Some(MinterResponse {
            minter: MINTER.to_string(),
            cap: Some(123456789u128.into()),
        }),
        token_marketing: Some(TokenMarketingInfo {
            project: Some(TOKEN_PROJECT_NAME.to_string()),
            description: Some(TOKEN_PROJECT_DESCRIPTION.to_string()),
            marketing_owner: Some(TOKEN_MARKETING_OWNER.to_string()),
            logo_url: Some(TOKEN_LOGO_URL.to_string()),
        }),
    });
    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: None,
            dao_membership_info: New(NewDaoMembershipMsg {
                membership_contract_code_id: CW20_CODE_ID,
                membership_info,
            }),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(result, Err(ZeroInitialWeightMember));

    Ok(())
}

#[test]
fn instantiate_new_token_membership_without_minter_sets_dao_as_minter() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let mut env = mock_env();
    env.contract.address = Addr::unchecked("dao_addr");
    let info = mock_info("sender", &[]);

    let membership_info = NewToken(NewTokenMembershipInfo {
        token_name: TOKEN_NAME.to_string(),
        token_symbol: TOKEN_SYMBOL.to_string(),
        token_decimals: TOKEN_DECIMALS,
        initial_token_balances: vec![],
        token_mint: None,
        token_marketing: None,
    });
    let response = instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        New(NewDaoMembershipMsg {
            membership_contract_code_id: CW20_CODE_ID,
            membership_info,
        }),
        None,
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::reply_on_success(
            wasm_instantiate(
                CW20_CODE_ID,
                &cw20_base::msg::InstantiateMsg {
                    name: TOKEN_NAME.to_string(),
                    symbol: TOKEN_SYMBOL.to_string(),
                    decimals: TOKEN_DECIMALS,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: "dao_addr".to_string(),
                        cap: None,
                    }),
                    marketing: None,
                },
                vec![],
                TOKEN_NAME.to_string(),
            )?,
            DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
        )]
    );

    Ok(())
}

#[test]
fn instantiate_new_nft_membership_instantiates_new_cw721_contract() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let membership_info = NewNft(NewNftMembershipInfo {
        nft_name: NFT_NAME.to_string(),
        nft_symbol: NFT_SYMBOL.to_string(),
        minter: Some(MINTER.to_string()),
    });
    let asset_whitelist = vec![
        AssetInfo::native("luna"),
        AssetInfo::cw20(Addr::unchecked("random_token")),
    ];
    let nft_whitelist = vec![Addr::unchecked("nft_addr1"), Addr::unchecked("nft_addr2")];
    let response = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config: stub_dao_gov_config(),
            dao_council: None,
            dao_membership_info: New(NewDaoMembershipMsg {
                membership_contract_code_id: CW721_CODE_ID,
                membership_info,
            }),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: Some(asset_whitelist.clone()),
            nft_whitelist: Some(nft_whitelist.clone()),
        },
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::reply_on_success(
            wasm_instantiate(
                CW721_CODE_ID,
                &cw721_base::msg::InstantiateMsg {
                    name: NFT_NAME.to_string(),
                    symbol: NFT_SYMBOL.to_string(),
                    minter: MINTER.to_string(),
                },
                vec![],
                "DAO NFT".to_string(),
            )?,
            DAO_MEMBERSHIP_CONTRACT_INSTANTIATE_REPLY_ID,
        )]
    );

    let asset_whitelist_response = query_asset_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(asset_whitelist_response.assets, asset_whitelist);

    let nft_whitelist_response = query_nft_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(nft_whitelist_response.nfts, nft_whitelist);

    // TODO: confirm that after reply() is called, the DAO membership contract is properly stored

    Ok(())
}

#[test]
fn instantiate_new_multisig_membership_stores_members_properly() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let dao_metadata = stub_dao_metadata();
    let dao_gov_config = DaoGovConfig {
        vote_duration: 105,
        threshold: Decimal::from_ratio(23u8, 100u8),
        unlocking_period: Duration::Height(1),
        ..stub_dao_gov_config()
    };

    let multisig_members = vec![
        MultisigMember {
            address: "member1".to_string(),
            weight: 200u64.into(),
        },
        MultisigMember {
            address: "member2".to_string(),
            weight: 400u64.into(),
        },
    ];
    let membership_info = NewMultisig(NewMultisigMembershipInfo { multisig_members });
    let asset_whitelist = vec![
        AssetInfo::native("uluna"),
        AssetInfo::cw20(Addr::unchecked("another_token")),
    ];
    let nft_whitelist = vec![Addr::unchecked("nft_addr1"), Addr::unchecked("nft_addr2")];
    instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata,
            dao_gov_config,
            dao_council: None,
            dao_membership_info: New(NewDaoMembershipMsg {
                membership_contract_code_id: CW3_FIXED_MULTISIG_CODE_ID,
                membership_info,
            }),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: Some(asset_whitelist.clone()),
            nft_whitelist: Some(nft_whitelist.clone()),
        },
    )?;

    let asset_whitelist_response = query_asset_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(asset_whitelist_response.assets, asset_whitelist);

    let nft_whitelist_response = query_nft_whitelist(mock_query_ctx(deps.as_ref(), &env))?;
    assert_eq!(nft_whitelist_response.nfts, nft_whitelist);

    let qctx = mock_query_ctx(deps.as_ref(), &env);

    assert_member_voting_power(&qctx, "member1", Decimal::from_ratio(1u8, 3u8));
    assert_member_voting_power(&qctx, "member2", Decimal::from_ratio(2u8, 3u8));
    assert_member_voting_power(&qctx, "member3", Decimal::zero());

    Ok(())
}

#[test]
fn instantiate_new_multisig_membership_with_zero_weight_member_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let multisig_members = vec![
        MultisigMember {
            address: "member1".to_string(),
            weight: 200u64.into(),
        },
        MultisigMember {
            address: "member2".to_string(),
            weight: 0u64.into(),
        },
        MultisigMember {
            address: "member3".to_string(),
            weight: 371u64.into(),
        },
    ];
    let membership_info = NewMultisig(NewMultisigMembershipInfo { multisig_members });
    let result = instantiate_stub_dao(
        deps.as_mut(),
        &env,
        &info,
        New(NewDaoMembershipMsg {
            membership_contract_code_id: CW3_FIXED_MULTISIG_CODE_ID,
            membership_info,
        }),
        None,
    );

    assert_eq!(result, Err(ZeroInitialWeightMember));

    Ok(())
}

#[test]
fn instantiate_dao_with_shorter_unstaking_than_voting_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let dao_gov_config = DaoGovConfig {
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::from_ratio(1u8, 10u8),
        vote_duration: 10u64,
        unlocking_period: Duration::Time(9u64),
        minimum_deposit: None,
    };

    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config,
            dao_council: None,
            dao_membership_info: stub_token_dao_membership_info(),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(result, Err(DaoError::VoteDurationLongerThanUnstaking {}));

    Ok(())
}

#[test]
fn instantiate_nft_dao_with_minimum_deposit_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let dao_gov_config = DaoGovConfig {
        minimum_deposit: Some(Uint128::one()),
        ..stub_dao_gov_config()
    };

    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config,
            dao_council: None,
            dao_membership_info: stub_nft_dao_membership_info(),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(result, Err(DaoError::MinimumDepositNotAllowed {}));

    Ok(())
}

#[test]
fn instantiate_multisig_dao_with_minimum_deposit_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    let dao_gov_config = DaoGovConfig {
        minimum_deposit: Some(Uint128::one()),
        ..stub_dao_gov_config()
    };

    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config,
            dao_council: None,
            dao_membership_info: stub_multisig_dao_membership_info(),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(result, Err(DaoError::MinimumDepositNotAllowed {}));

    Ok(())
}

#[test]
fn instantiate_dao_with_quorum_over_one_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    let dao_gov_config = DaoGovConfig {
        quorum: Decimal::from_ratio(1001u64, 1000u64),
        threshold: Decimal::from_ratio(1u8, 10u8),
        vote_duration: 10u64,
        unlocking_period: Duration::Time(10u64),
        minimum_deposit: None,
    };

    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config,
            dao_council: None,
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(
        result,
        Err(InvalidArgument {
            msg: "Invalid quorum, must be 0 < quorum <= 1".to_string()
        })
    );

    Ok(())
}

#[test]
fn instantiate_dao_with_quorum_of_zero_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    let dao_gov_config = DaoGovConfig {
        quorum: Decimal::zero(),
        threshold: Decimal::from_ratio(1u8, 10u8),
        vote_duration: 10u64,
        unlocking_period: Duration::Time(10u64),
        minimum_deposit: None,
    };

    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config,
            dao_council: None,
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(
        result,
        Err(InvalidArgument {
            msg: "Invalid quorum, must be 0 < quorum <= 1".to_string()
        })
    );

    Ok(())
}

#[test]
fn instantiate_dao_with_threshold_over_one_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    let dao_gov_config = DaoGovConfig {
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::from_ratio(1001u64, 1000u64),
        vote_duration: 10u64,
        unlocking_period: Duration::Time(10u64),
        minimum_deposit: None,
    };

    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config,
            dao_council: None,
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(
        result,
        Err(InvalidArgument {
            msg: "Invalid threshold, must be 0 < threshold <= 1".to_string()
        })
    );

    Ok(())
}

#[test]
fn instantiate_dao_with_threshold_of_zero_fails() -> DaoResult<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    deps.querier
        .with_token_infos(&[(CW20_ADDR, &stub_token_info())]);

    let dao_gov_config = DaoGovConfig {
        quorum: Decimal::from_ratio(1u8, 10u8),
        threshold: Decimal::zero(),
        vote_duration: 10u64,
        unlocking_period: Duration::Time(10u64),
        minimum_deposit: None,
    };

    let result = instantiate(
        deps.as_mut(),
        env.clone(),
        info,
        InstantiateMsg {
            dao_metadata: stub_dao_metadata(),
            dao_gov_config,
            dao_council: None,
            dao_membership_info: existing_token_dao_membership(CW20_ADDR),
            enterprise_factory_contract: stub_enterprise_factory_contract(),
            asset_whitelist: None,
            nft_whitelist: None,
        },
    );

    assert_eq!(
        result,
        Err(InvalidArgument {
            msg: "Invalid threshold, must be 0 < threshold <= 1".to_string()
        })
    );

    Ok(())
}
