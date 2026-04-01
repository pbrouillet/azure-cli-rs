mod auth;
mod cli;
mod cloud;
mod commands;
mod config;
mod error;
#[allow(dead_code)]
mod generated;
pub mod http_client;
mod profile;
mod arm;
mod output;
mod rest;
mod selector;
#[cfg(test)]
pub mod testing;

use clap::Parser;
use cli::{
    AccountCommands, AppserviceCommands, AppservicePlanCommands, AppserviceAseCommands,
    AppserviceDomainCommands, AppservicePlanIdentityCommands,
    Cli, Commands, ConfigCommands,
    DeploymentCommands, DeploymentOperationCommands, DeploymentScopeCommands,
    DeploymentScriptsCommands, FeatureCommands, FeatureRegistrationCommands, GroupCommands,
    KeyvaultCommands, KeyvaultSecretCommands, LockCommands, ManagedappCommands,
    ManagedappDefinitionCommands, ManagementGroupCommands, ManagementGroupEntitiesCommands,
    ManagementGroupHierarchySettingsCommands, ManagementGroupSubscriptionCommands,
    ManagementGroupTenantBackfillCommands, ProviderCommands, ProviderOperationCommands,
    ProviderPermissionCommands, ResourceCommands, ResourceLinkCommands, StackCommands,
    StackScopeCommands, StaticwebappAppsettingsCommands, StaticwebappCommands,
    StaticwebappEnvironmentCommands, StaticwebappHostnameCommands, StorageAccountCommands,
    StorageCommands, TagCommands, TsCommands,
    WebappCommands, WebappConfigAccessRestrictionCommands, WebappConfigAppsettingsCommands,
    WebappConfigBackupCommands, WebappConfigCommands, WebappConfigConnstrCommands,
    WebappConfigContainerCommands, WebappConfigHostnameCommands, WebappConfigSslCommands,
    WebappCorsCommands, WebappDeploymentCommands, WebappDeploymentContainerCommands,
    WebappDeploymentGithubActionsCommands, WebappDeploymentSlotCommands,
    WebappDeploymentSourceCommands, WebappDeploymentUserCommands, WebappIdentityCommands,
    FunctionappCommands, FunctionappConfigCommands, FunctionappConfigAppsettingsCommands,
    FunctionappKeysCommands, FunctionappFunctionCommands, FunctionappFunctionKeysCommands,
    FunctionappDeploymentCommands, FunctionappDeploymentSourceCommands,
    FunctionappPlanCommands, FunctionappDeploymentSlotCommands,
    FunctionappVnetIntegrationCommands, FunctionappScaleConfigCommands,
};

type CmdResult = error::Result<Option<serde_json::Value>>;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Apply global flags
    commands::set_debug(cli.global.debug);
    commands::set_subscription_override(cli.global.subscription.clone());

    // Resolve output format: CLI flag > config default > json
    let output_format = {
        // clap always provides a default, so check if user explicitly passed -o
        // For simplicity, just use the CLI value (config default is a future enhancement)
        cli.global.output
    };
    let query = cli.global.query.clone();

    let result: CmdResult = match cli.command {
        Commands::Login(args) => cmd_handlers::login(args).await,
        Commands::Logout(args) => cmd_handlers::logout(args).await,
        Commands::Account(sub) => match sub {
            AccountCommands::Show => cmd_handlers::account_show().await,
            AccountCommands::List(args) => cmd_handlers::account_list(args).await,
            AccountCommands::Set(args) => cmd_handlers::account_set(args).await,
            AccountCommands::GetAccessToken(args) => cmd_handlers::account_get_access_token(args).await,
            AccountCommands::ManagementGroup(mg_sub) => match mg_sub {
                ManagementGroupCommands::List => {
                    cmd_handlers::wrap_list(crate::commands::management_group::list().await)
                }
                ManagementGroupCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::management_group::show(&args.name, args.expand.as_deref(), args.recurse).await)
                }
                ManagementGroupCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::management_group::create(&args.name, args.display_name.as_deref(), args.parent.as_deref()).await)
                }
                ManagementGroupCommands::Delete(args) => {
                    cmd_handlers::wrap_none(crate::commands::management_group::delete(&args.name).await)
                }
                ManagementGroupCommands::CheckNameAvailability(args) => {
                    cmd_handlers::wrap(crate::commands::management_group::check_name_availability(&args.name).await)
                }
                ManagementGroupCommands::Subscription(sub_cmd) => match sub_cmd {
                    ManagementGroupSubscriptionCommands::Add(args) => {
                        cmd_handlers::wrap(crate::commands::management_group::subscription_add(&args.name, &args.subscription).await)
                    }
                    ManagementGroupSubscriptionCommands::Remove(args) => {
                        cmd_handlers::wrap_none(crate::commands::management_group::subscription_remove(&args.name, &args.subscription).await)
                    }
                    ManagementGroupSubscriptionCommands::ShowSubUnderMg(args) => {
                        cmd_handlers::wrap(crate::commands::management_group::subscription_show(&args.name, &args.subscription).await)
                    }
                },
                ManagementGroupCommands::Entities(ent) => match ent {
                    ManagementGroupEntitiesCommands::List => {
                        cmd_handlers::wrap_list(crate::commands::management_group::entities_list().await)
                    }
                },
                ManagementGroupCommands::HierarchySettings(hs) => match hs {
                    ManagementGroupHierarchySettingsCommands::List(args) => {
                        cmd_handlers::wrap(crate::commands::management_group::hierarchy_settings_list(&args.name).await)
                    }
                    ManagementGroupHierarchySettingsCommands::Create(args) => {
                        cmd_handlers::wrap(crate::commands::management_group::hierarchy_settings_create(&args.name, args.require_authorization, args.default_management_group.as_deref()).await)
                    }
                    ManagementGroupHierarchySettingsCommands::Delete(args) => {
                        cmd_handlers::wrap_none(crate::commands::management_group::hierarchy_settings_delete(&args.name).await)
                    }
                },
                ManagementGroupCommands::TenantBackfill(tb) => match tb {
                    ManagementGroupTenantBackfillCommands::Get => {
                        cmd_handlers::wrap(crate::commands::management_group::tenant_backfill_get().await)
                    }
                    ManagementGroupTenantBackfillCommands::Start => {
                        cmd_handlers::wrap(crate::commands::management_group::tenant_backfill_start().await)
                    }
                },
            },
            AccountCommands::ListLocations => {
                cmd_handlers::wrap(cmd_handlers::list_locations().await)
            }
        },
        Commands::Rest(args) => cmd_handlers::rest(args).await,
        Commands::Completions(args) => {
            clap_complete::generate(
                args.shell,
                &mut <cli::Cli as clap::CommandFactory>::command(),
                "azrs",
                &mut std::io::stdout(),
            );
            Ok(None)
        }
        Commands::Group(sub) => match sub {
            GroupCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::group::create(&args.name, &args.location, args.tags.as_deref()).await)
            }
            GroupCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::group::list(args.tag.as_deref()).await)
            }
            GroupCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::group::show(&args.name).await)
            }
            GroupCommands::Delete(args) => {
                if !args.yes && !cmd_handlers::confirm(&format!("delete resource group '{}'", args.name)) {
                    return;
                }
                cmd_handlers::wrap_none(crate::commands::group::delete(&args.name).await)
            }
            GroupCommands::Exists(args) => {
                cmd_handlers::wrap(crate::commands::group::exists(&args.name).await)
            }
            GroupCommands::Update(args) => {
                cmd_handlers::wrap(crate::commands::group::update(&args.name, args.tags.as_deref()).await)
            }
            GroupCommands::Export(args) => {
                cmd_handlers::wrap(crate::commands::group::export(&args.name).await)
            }
            GroupCommands::Wait(args) => {
                cmd_handlers::wrap_none(crate::commands::group::wait(
                    &args.name,
                    args.created,
                    args.updated,
                    args.deleted,
                    args.exists,
                    args.custom.as_deref(),
                    args.interval,
                    args.timeout,
                ).await)
            }
            GroupCommands::Lock(lock_sub) => match lock_sub {
                LockCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::lock::list(args.resource_group.as_deref()).await)
                }
                LockCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::lock::create(&args.name, &args.lock_type, args.resource_group.as_deref(), args.notes.as_deref()).await)
                }
                LockCommands::Delete(args) => {
                    cmd_handlers::wrap_none(crate::commands::lock::delete(&args.name, args.resource_group.as_deref()).await)
                }
                LockCommands::Update(args) => {
                    cmd_handlers::wrap(crate::commands::lock::update(&args.name, args.resource_group.as_deref(), args.lock_type.as_deref(), args.notes.as_deref()).await)
                }
            },
        },
        Commands::Resource(sub) => match sub {
            ResourceCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::resource::list(
                    args.resource_group.as_deref(),
                    args.resource_type.as_deref(),
                    args.tag.as_deref(),
                    args.name.as_deref(),
                ).await)
            }
            ResourceCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::resource::show(
                    args.ids.as_deref(),
                    args.resource_group.as_deref(),
                    args.namespace.as_deref(),
                    args.resource_type.as_deref(),
                    args.name.as_deref(),
                    args.parent.as_deref(),
                    &args.api_version,
                ).await)
            }
            ResourceCommands::Delete(args) => {
                if !args.yes && !cmd_handlers::confirm("delete this resource") {
                    return;
                }
                cmd_handlers::wrap_none(crate::commands::resource::delete(
                    args.ids.as_deref(),
                    args.resource_group.as_deref(),
                    args.namespace.as_deref(),
                    args.resource_type.as_deref(),
                    args.name.as_deref(),
                    args.parent.as_deref(),
                    &args.api_version,
                ).await)
            }
            ResourceCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::resource::create(
                    args.ids.as_deref(),
                    args.resource_group.as_deref(),
                    args.namespace.as_deref(),
                    args.resource_type.as_deref(),
                    args.name.as_deref(),
                    args.parent.as_deref(),
                    &args.api_version,
                    &args.properties,
                    args.location.as_deref(),
                    args.tags.as_deref(),
                ).await)
            }
            ResourceCommands::Update(args) => {
                cmd_handlers::wrap(crate::commands::resource::update(
                    args.ids.as_deref(),
                    args.resource_group.as_deref(),
                    args.namespace.as_deref(),
                    args.resource_type.as_deref(),
                    args.name.as_deref(),
                    args.parent.as_deref(),
                    &args.api_version,
                    args.set.as_deref(),
                ).await)
            }
            ResourceCommands::Tag(args) => {
                cmd_handlers::wrap(crate::commands::resource::tag(
                    args.ids.as_deref(),
                    args.resource_group.as_deref(),
                    args.namespace.as_deref(),
                    args.resource_type.as_deref(),
                    args.name.as_deref(),
                    args.parent.as_deref(),
                    &args.api_version,
                    &args.tags,
                    args.incremental,
                ).await)
            }
            ResourceCommands::InvokeAction(args) => {
                cmd_handlers::wrap(crate::commands::resource::invoke_action(
                    args.ids.as_deref(),
                    args.resource_group.as_deref(),
                    args.namespace.as_deref(),
                    args.resource_type.as_deref(),
                    args.name.as_deref(),
                    args.parent.as_deref(),
                    &args.api_version,
                    &args.action,
                    args.request_body.as_deref(),
                ).await)
            }
            ResourceCommands::Wait(args) => {
                cmd_handlers::wrap_none(crate::commands::resource::wait(
                    args.ids.as_deref(),
                    args.resource_group.as_deref(),
                    args.namespace.as_deref(),
                    args.resource_type.as_deref(),
                    args.name.as_deref(),
                    args.parent.as_deref(),
                    &args.api_version,
                    args.created,
                    args.updated,
                    args.deleted,
                    args.exists,
                    args.custom.as_deref(),
                    args.interval,
                    args.timeout,
                ).await)
            }
            ResourceCommands::Lock(lock_sub) => match lock_sub {
                LockCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::lock::list(args.resource_group.as_deref()).await)
                }
                LockCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::lock::create(&args.name, &args.lock_type, args.resource_group.as_deref(), args.notes.as_deref()).await)
                }
                LockCommands::Delete(args) => {
                    cmd_handlers::wrap_none(crate::commands::lock::delete(&args.name, args.resource_group.as_deref()).await)
                }
                LockCommands::Update(args) => {
                    cmd_handlers::wrap(crate::commands::lock::update(&args.name, args.resource_group.as_deref(), args.lock_type.as_deref(), args.notes.as_deref()).await)
                }
            },
            ResourceCommands::Link(link_sub) => match link_sub {
                ResourceLinkCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::resource::link_list(args.scope.as_deref()).await)
                }
                ResourceLinkCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::resource::link_show(&args.link_id).await)
                }
                ResourceLinkCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::resource::link_create(&args.link_id, &args.target_id, args.notes.as_deref()).await)
                }
                ResourceLinkCommands::Delete(args) => {
                    cmd_handlers::wrap_none(crate::commands::resource::link_delete(&args.link_id).await)
                }
                ResourceLinkCommands::Update(args) => {
                    cmd_handlers::wrap(crate::commands::resource::link_update(&args.link_id, args.target_id.as_deref(), args.notes.as_deref()).await)
                }
            },
        },
        Commands::Provider(sub) => match sub {
            ProviderCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::provider::list(args.expand.as_deref()).await)
            }
            ProviderCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::provider::show(&args.namespace, args.expand.as_deref()).await)
            }
            ProviderCommands::Register(args) => {
                cmd_handlers::wrap(crate::commands::provider::register(&args.namespace).await)
            }
            ProviderCommands::Unregister(args) => {
                cmd_handlers::wrap(crate::commands::provider::unregister(&args.namespace).await)
            }
            ProviderCommands::Operation(op) => match op {
                ProviderOperationCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::provider::operation_list(args.namespace.as_deref()).await)
                }
            },
            ProviderCommands::Permission(perm) => match perm {
                ProviderPermissionCommands::List(args) => {
                    cmd_handlers::wrap(crate::commands::provider::permission_list(&args.namespace).await)
                }
            },
        },
        Commands::Feature(sub) => match sub {
            FeatureCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::feature::list(args.namespace.as_deref()).await)
            }
            FeatureCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::feature::show(&args.namespace, &args.name).await)
            }
            FeatureCommands::Register(args) => {
                cmd_handlers::wrap(crate::commands::feature::register(&args.namespace, &args.name).await)
            }
            FeatureCommands::Unregister(args) => {
                cmd_handlers::wrap(crate::commands::feature::unregister(&args.namespace, &args.name).await)
            }
            FeatureCommands::Registration(reg) => match reg {
                FeatureRegistrationCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::feature::registration_list(args.namespace.as_deref()).await)
                }
                FeatureRegistrationCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::feature::registration_show(&args.namespace, &args.name).await)
                }
                FeatureRegistrationCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::feature::registration_create(&args.namespace, &args.name).await)
                }
                FeatureRegistrationCommands::Delete(args) => {
                    cmd_handlers::wrap_none(crate::commands::feature::registration_delete(&args.namespace, &args.name).await)
                }
            },
        },
        Commands::Tag(sub) => match sub {
            TagCommands::List => {
                cmd_handlers::wrap_list(crate::commands::tag::list().await)
            }
            TagCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::tag::create(
                    args.resource_id.as_deref(),
                    args.tags.as_deref(),
                    args.name.as_deref(),
                ).await)
            }
            TagCommands::Delete(args) => {
                if !args.yes && !cmd_handlers::confirm("delete tags") {
                    return;
                }
                cmd_handlers::wrap_none(crate::commands::tag::delete(
                    args.resource_id.as_deref(),
                    args.name.as_deref(),
                ).await)
            }
            TagCommands::Update(args) => {
                cmd_handlers::wrap(crate::commands::tag::update(
                    &args.resource_id,
                    &args.operation,
                    &args.tags,
                ).await)
            }
            TagCommands::AddValue(args) => {
                cmd_handlers::wrap(crate::commands::tag::add_value(&args.name, &args.value).await)
            }
            TagCommands::RemoveValue(args) => {
                cmd_handlers::wrap_none(crate::commands::tag::remove_value(&args.name, &args.value).await)
            }
        },
        Commands::Lock(sub) => match sub {
            LockCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::lock::list(args.resource_group.as_deref()).await)
            }
            LockCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::lock::create(
                    &args.name,
                    &args.lock_type,
                    args.resource_group.as_deref(),
                    args.notes.as_deref(),
                ).await)
            }
            LockCommands::Delete(args) => {
                cmd_handlers::wrap_none(crate::commands::lock::delete(
                    &args.name,
                    args.resource_group.as_deref(),
                ).await)
            }
            LockCommands::Update(args) => {
                cmd_handlers::wrap(crate::commands::lock::update(
                    &args.name,
                    args.resource_group.as_deref(),
                    args.lock_type.as_deref(),
                    args.notes.as_deref(),
                ).await)
            }
        },
        Commands::Deployment(sub) => {
            use crate::commands::deployment::Scope;
            match sub {
                DeploymentCommands::Group(scope_sub) => {
                    let rg = scope_sub.resource_group_ref().unwrap_or("").to_owned();
                    dispatch_deployment_scope(scope_sub, Scope::ResourceGroup(&rg)).await
                }
                DeploymentCommands::Sub(scope_sub) => {
                    dispatch_deployment_scope(scope_sub, Scope::Subscription).await
                }
                DeploymentCommands::Mg(scope_sub) => {
                    let mg_id = scope_sub.management_group_ref().unwrap_or("").to_owned();
                    dispatch_deployment_scope(scope_sub, Scope::ManagementGroup(&mg_id)).await
                }
                DeploymentCommands::Tenant(scope_sub) => {
                    dispatch_deployment_scope(scope_sub, Scope::Tenant).await
                }
            }
        },
        Commands::DeploymentScripts(sub) => match sub {
            DeploymentScriptsCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::deployment::scripts_list(args.resource_group.as_deref()).await)
            }
            DeploymentScriptsCommands::ShowLog(args) => {
                cmd_handlers::wrap(crate::commands::deployment::scripts_show_log(&args.resource_group, &args.name).await)
            }
            DeploymentScriptsCommands::Delete(args) => {
                cmd_handlers::wrap_none(crate::commands::deployment::scripts_delete(&args.resource_group, &args.name).await)
            }
        },
        Commands::Ts(sub) => match sub {
            TsCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::template_specs::list(args.resource_group.as_deref()).await)
            }
            TsCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::template_specs::show(&args.resource_group, &args.name, args.version.as_deref()).await)
            }
            TsCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::template_specs::create(
                    &args.resource_group,
                    &args.name,
                    &args.version,
                    &args.template_file,
                    &args.location,
                    args.description.as_deref(),
                    args.display_name.as_deref(),
                    args.tags.as_deref(),
                ).await)
            }
            TsCommands::Delete(args) => {
                cmd_handlers::wrap_none(crate::commands::template_specs::delete(&args.resource_group, &args.name, args.version.as_deref()).await)
            }
            TsCommands::Export(args) => {
                cmd_handlers::wrap(crate::commands::template_specs::export(&args.resource_group, &args.name, &args.version, &args.output_folder).await)
            }
        },
        Commands::Stack(sub) => {
            use crate::commands::stack::StackScope;
            match sub {
                StackCommands::Group(scope_sub) => {
                    let rg = scope_sub.resource_group_ref().unwrap_or("").to_owned();
                    dispatch_stack_scope(scope_sub, StackScope::ResourceGroup(&rg)).await
                }
                StackCommands::Sub(scope_sub) => {
                    dispatch_stack_scope(scope_sub, StackScope::Subscription).await
                }
                StackCommands::Mg(scope_sub) => {
                    let mg_id = scope_sub.management_group_ref().unwrap_or("").to_owned();
                    dispatch_stack_scope(scope_sub, StackScope::ManagementGroup(&mg_id)).await
                }
            }
        },
        Commands::Managedapp(sub) => match sub {
            ManagedappCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::managed_app::list(args.resource_group.as_deref()).await)
            }
            ManagedappCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::managed_app::show(&args.resource_group, &args.name).await)
            }
            ManagedappCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::managed_app::create(
                    &args.resource_group,
                    &args.name,
                    &args.kind,
                    &args.managed_rg_id,
                    &args.location,
                    args.definition_id.as_deref(),
                    args.parameters.as_deref(),
                ).await)
            }
            ManagedappCommands::Delete(args) => {
                cmd_handlers::wrap_none(crate::commands::managed_app::delete(&args.resource_group, &args.name).await)
            }
            ManagedappCommands::Definition(def_sub) => match def_sub {
                ManagedappDefinitionCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::managed_app::definition_list(&args.resource_group).await)
                }
                ManagedappDefinitionCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::managed_app::definition_create(
                        &args.resource_group,
                        &args.name,
                        &args.lock_level,
                        &args.location,
                        args.display_name.as_deref(),
                        args.description.as_deref(),
                        args.package_file_uri.as_deref(),
                        args.authorizations.as_deref(),
                    ).await)
                }
                ManagedappDefinitionCommands::Delete(args) => {
                    cmd_handlers::wrap_none(crate::commands::managed_app::definition_delete(&args.resource_group, &args.name).await)
                }
                ManagedappDefinitionCommands::Update(args) => {
                    cmd_handlers::wrap(crate::commands::managed_app::definition_update(
                        &args.resource_group,
                        &args.name,
                        args.lock_level.as_deref(),
                        args.display_name.as_deref(),
                        args.description.as_deref(),
                        args.tags.as_deref(),
                    ).await)
                }
            },
        },
        Commands::Webapp(sub) => match sub {
            WebappCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::webapp::list(args.resource_group.as_deref()).await)
            }
            WebappCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::webapp::show(&args.resource_group, &args.name).await)
            }
            WebappCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::webapp::create(
                    &args.resource_group,
                    &args.name,
                    &args.plan,
                    args.runtime.as_deref(),
                    args.startup_file.as_deref(),
                    args.deployment_container_image_name.as_deref(),
                    args.tags.as_deref(),
                ).await)
            }
            WebappCommands::Delete(args) => {
                if !args.yes && !cmd_handlers::confirm(&format!("delete web app '{}'", args.name)) {
                    return;
                }
                cmd_handlers::wrap_none(crate::commands::webapp::delete(&args.resource_group, &args.name).await)
            }
            WebappCommands::Stop(args) => {
                cmd_handlers::wrap_none(crate::commands::webapp::stop(&args.resource_group, &args.name).await)
            }
            WebappCommands::Start(args) => {
                cmd_handlers::wrap_none(crate::commands::webapp::start(&args.resource_group, &args.name).await)
            }
            WebappCommands::Restart(args) => {
                cmd_handlers::wrap_none(crate::commands::webapp::restart(&args.resource_group, &args.name).await)
            }
            WebappCommands::Update(args) => {
                cmd_handlers::wrap(crate::commands::webapp::update(&args.resource_group, &args.name, args.set.as_deref()).await)
            }
            WebappCommands::ListRuntimes(args) => {
                cmd_handlers::wrap(crate::commands::webapp::list_runtimes(args.os.as_deref()).await)
            }
            WebappCommands::Deploy(args) => {
                cmd_handlers::wrap(crate::commands::webapp::deploy(&args.resource_group, &args.name, &args.src_path, args.deploy_type.as_deref()).await)
            }
            WebappCommands::Identity(id_sub) => match id_sub {
                WebappIdentityCommands::Assign(args) => {
                    cmd_handlers::wrap(crate::commands::webapp::identity_assign(&args.resource_group, &args.name, args.identity_type.as_deref()).await)
                }
                WebappIdentityCommands::Remove(args) => {
                    cmd_handlers::wrap(crate::commands::webapp::identity_remove(&args.resource_group, &args.name).await)
                }
            },
            WebappCommands::Cors(cors_sub) => match cors_sub {
                WebappCorsCommands::Add(args) => {
                    cmd_handlers::wrap(crate::commands::webapp::cors_add(&args.resource_group, &args.name, &args.allowed_origins).await)
                }
                WebappCorsCommands::Remove(args) => {
                    cmd_handlers::wrap(crate::commands::webapp::cors_remove(&args.resource_group, &args.name, &args.allowed_origins).await)
                }
            },
            WebappCommands::Config(cfg_sub) => match cfg_sub {
                WebappConfigCommands::Set(args) => {
                    cmd_handlers::wrap(crate::commands::webapp::config_set(&args.resource_group, &args.name, &args.set).await)
                }
                WebappConfigCommands::Appsettings(as_sub) => match as_sub {
                    WebappConfigAppsettingsCommands::List(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_appsettings_list(&args.resource_group, &args.name).await)
                    }
                    WebappConfigAppsettingsCommands::Set(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_appsettings_set(&args.resource_group, &args.name, &args.settings).await)
                    }
                    WebappConfigAppsettingsCommands::Delete(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_appsettings_delete(&args.resource_group, &args.name, &args.setting_names).await)
                    }
                },
                WebappConfigCommands::ConnectionString(cs_sub) => match cs_sub {
                    WebappConfigConnstrCommands::List(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_connstr_list(&args.resource_group, &args.name).await)
                    }
                    WebappConfigConnstrCommands::Set(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_connstr_set(&args.resource_group, &args.name, &args.settings, &args.connection_string_type).await)
                    }
                    WebappConfigConnstrCommands::Delete(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_connstr_delete(&args.resource_group, &args.name, &args.setting_names).await)
                    }
                },
                WebappConfigCommands::Hostname(hn_sub) => match hn_sub {
                    WebappConfigHostnameCommands::List(args) => {
                        cmd_handlers::wrap_list(crate::commands::webapp::config_hostname_list(&args.resource_group, &args.name).await)
                    }
                    WebappConfigHostnameCommands::Add(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_hostname_add(&args.resource_group, &args.name, &args.hostname).await)
                    }
                    WebappConfigHostnameCommands::Delete(args) => {
                        cmd_handlers::wrap_none(crate::commands::webapp::config_hostname_delete(&args.resource_group, &args.name, &args.hostname).await)
                    }
                },
                WebappConfigCommands::Ssl(ssl_sub) => match ssl_sub {
                    WebappConfigSslCommands::List(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_ssl_list(&args.resource_group, &args.name).await)
                    }
                    WebappConfigSslCommands::Bind(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_ssl_bind(&args.resource_group, &args.name, &args.ssl_type, &args.certificate_thumbprint, &args.hostname).await)
                    }
                    WebappConfigSslCommands::Unbind(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_ssl_unbind(&args.resource_group, &args.name, &args.hostname).await)
                    }
                },
                WebappConfigCommands::AccessRestriction(ar_sub) => match ar_sub {
                    WebappConfigAccessRestrictionCommands::Add(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_access_restriction_add(&args.resource_group, &args.name, &args.rule_name, args.priority, &args.action, args.ip_address.as_deref()).await)
                    }
                    WebappConfigAccessRestrictionCommands::Remove(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_access_restriction_remove(&args.resource_group, &args.name, &args.rule_name).await)
                    }
                    WebappConfigAccessRestrictionCommands::Set(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_access_restriction_set(&args.resource_group, &args.name, args.use_same_restrictions_for_scm_site).await)
                    }
                },
                WebappConfigCommands::Container(ct_sub) => match ct_sub {
                    WebappConfigContainerCommands::Set(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_container_set(&args.resource_group, &args.name, &args.docker_custom_image_name, args.docker_registry_server_url.as_deref(), args.docker_registry_server_user.as_deref(), args.docker_registry_server_password.as_deref()).await)
                    }
                    WebappConfigContainerCommands::Delete(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_container_delete(&args.resource_group, &args.name).await)
                    }
                },
                WebappConfigCommands::Backup(bk_sub) => match bk_sub {
                    WebappConfigBackupCommands::List(args) => {
                        cmd_handlers::wrap_list(crate::commands::webapp::config_backup_list(&args.resource_group, &args.name).await)
                    }
                    WebappConfigBackupCommands::Create(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::config_backup_create(&args.resource_group, &args.name, &args.backup_name, &args.storage_account_url).await)
                    }
                    WebappConfigBackupCommands::Delete(args) => {
                        cmd_handlers::wrap_none(crate::commands::webapp::config_backup_delete(&args.resource_group, &args.name, &args.backup_id).await)
                    }
                    WebappConfigBackupCommands::Restore(args) => {
                        cmd_handlers::wrap_none(crate::commands::webapp::config_backup_restore(&args.resource_group, &args.name, &args.backup_id, &args.storage_account_url).await)
                    }
                },
            },
            WebappCommands::Deployment(dep_sub) => match dep_sub {
                WebappDeploymentCommands::ListPublishingProfiles(args) => {
                    cmd_handlers::wrap(crate::commands::webapp::deployment_list_publishing_profiles(&args.resource_group, &args.name).await)
                }
                WebappDeploymentCommands::Source(src_sub) => match src_sub {
                    WebappDeploymentSourceCommands::Show(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::deployment_source_show(&args.resource_group, &args.name).await)
                    }
                    WebappDeploymentSourceCommands::ConfigZip(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::deployment_source_config_zip(&args.resource_group, &args.name, &args.src).await)
                    }
                    WebappDeploymentSourceCommands::Delete(args) => {
                        cmd_handlers::wrap_none(crate::commands::webapp::deployment_source_delete(&args.resource_group, &args.name).await)
                    }
                    WebappDeploymentSourceCommands::Sync(args) => {
                        cmd_handlers::wrap_none(crate::commands::webapp::deployment_source_sync(&args.resource_group, &args.name).await)
                    }
                },
                WebappDeploymentCommands::Slot(slot_sub) => match slot_sub {
                    WebappDeploymentSlotCommands::List(args) => {
                        cmd_handlers::wrap_list(crate::commands::webapp::deployment_slot_list(&args.resource_group, &args.name).await)
                    }
                    WebappDeploymentSlotCommands::Create(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::deployment_slot_create(&args.resource_group, &args.name, &args.slot).await)
                    }
                    WebappDeploymentSlotCommands::Delete(args) => {
                        cmd_handlers::wrap_none(crate::commands::webapp::deployment_slot_delete(&args.resource_group, &args.name, &args.slot).await)
                    }
                    WebappDeploymentSlotCommands::Swap(args) => {
                        cmd_handlers::wrap_none(crate::commands::webapp::deployment_slot_swap(&args.resource_group, &args.name, &args.slot, &args.target_slot).await)
                    }
                },
                WebappDeploymentCommands::GithubActions(ga_sub) => match ga_sub {
                    WebappDeploymentGithubActionsCommands::Add(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::deployment_github_actions_add(&args.resource_group, &args.name, &args.repo, &args.branch, &args.token).await)
                    }
                    WebappDeploymentGithubActionsCommands::Remove(args) => {
                        cmd_handlers::wrap_none(crate::commands::webapp::deployment_github_actions_remove(&args.resource_group, &args.name).await)
                    }
                },
                WebappDeploymentCommands::Container(dc_sub) => match dc_sub {
                    WebappDeploymentContainerCommands::Config(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::deployment_container_config(&args.resource_group, &args.name, args.enable_cd).await)
                    }
                    WebappDeploymentContainerCommands::ShowCdUrl(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::deployment_container_show_cd_url(&args.resource_group, &args.name).await)
                    }
                },
                WebappDeploymentCommands::User(user_sub) => match user_sub {
                    WebappDeploymentUserCommands::Set(args) => {
                        cmd_handlers::wrap(crate::commands::webapp::deployment_user_set(&args.user_name, &args.password).await)
                    }
                },
            },
        },
        Commands::Functionapp(sub) => match sub {
            FunctionappCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::functionapp::list(args.resource_group.as_deref()).await)
            }
            FunctionappCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::functionapp::show(&args.resource_group, &args.name).await)
            }
            FunctionappCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::functionapp::create(
                    &args.resource_group,
                    &args.name,
                    args.plan.as_deref(),
                    args.consumption_plan_location.is_some(),
                    args.runtime.as_deref(),
                    args.os_type.as_deref(),
                    args.storage_account.as_deref(),
                    args.consumption_plan_location.as_deref().or(args.location.as_deref()),
                    args.tags.as_deref(),
                ).await)
            }
            FunctionappCommands::Delete(args) => {
                if !args.yes && !cmd_handlers::confirm(&format!("delete function app '{}'", args.name)) {
                    return;
                }
                cmd_handlers::wrap_none(crate::commands::functionapp::delete(&args.resource_group, &args.name).await)
            }
            FunctionappCommands::Stop(args) => {
                cmd_handlers::wrap_none(crate::commands::functionapp::stop(&args.resource_group, &args.name).await)
            }
            FunctionappCommands::Start(args) => {
                cmd_handlers::wrap_none(crate::commands::functionapp::start(&args.resource_group, &args.name).await)
            }
            FunctionappCommands::Restart(args) => {
                cmd_handlers::wrap_none(crate::commands::functionapp::restart(&args.resource_group, &args.name).await)
            }
            FunctionappCommands::Update(args) => {
                cmd_handlers::wrap(crate::commands::functionapp::update(&args.resource_group, &args.name, args.set.as_deref()).await)
            }
            FunctionappCommands::ListRuntimes => {
                cmd_handlers::wrap(crate::commands::functionapp::list_runtimes().await)
            }
            FunctionappCommands::Deploy(args) => {
                cmd_handlers::wrap(crate::commands::functionapp::deploy(&args.resource_group, &args.name, &args.src_path, args.deploy_type.as_deref()).await)
            }
            FunctionappCommands::Config(cfg_sub) => match cfg_sub {
                FunctionappConfigCommands::Appsettings(as_sub) => match as_sub {
                    FunctionappConfigAppsettingsCommands::List(args) => {
                        cmd_handlers::wrap(crate::commands::functionapp::config_appsettings_list(&args.resource_group, &args.name).await)
                    }
                    FunctionappConfigAppsettingsCommands::Set(args) => {
                        cmd_handlers::wrap(crate::commands::functionapp::config_appsettings_set(&args.resource_group, &args.name, &args.settings).await)
                    }
                    FunctionappConfigAppsettingsCommands::Delete(args) => {
                        cmd_handlers::wrap(crate::commands::functionapp::config_appsettings_delete(&args.resource_group, &args.name, &args.setting_names).await)
                    }
                },
            },
            FunctionappCommands::Keys(keys_sub) => match keys_sub {
                FunctionappKeysCommands::List(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp::keys_list(&args.resource_group, &args.name).await)
                }
                FunctionappKeysCommands::Set(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp::keys_set(&args.resource_group, &args.name, &args.key_name, args.key_value.as_deref()).await)
                }
                FunctionappKeysCommands::Delete(args) => {
                    if !args.yes && !cmd_handlers::confirm(&format!("delete key '{}'", args.key_name)) {
                        return;
                    }
                    cmd_handlers::wrap_none(crate::commands::functionapp::keys_delete(&args.resource_group, &args.name, &args.key_name).await)
                }
            },
            FunctionappCommands::Function(fn_sub) => match fn_sub {
                FunctionappFunctionCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::functionapp::function_list(&args.resource_group, &args.name).await)
                }
                FunctionappFunctionCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp::function_show(&args.resource_group, &args.name, &args.function_name).await)
                }
                FunctionappFunctionCommands::Delete(args) => {
                    if !args.yes && !cmd_handlers::confirm(&format!("delete function '{}'", args.function_name)) {
                        return;
                    }
                    cmd_handlers::wrap_none(crate::commands::functionapp::function_delete(&args.resource_group, &args.name, &args.function_name).await)
                }
                FunctionappFunctionCommands::Keys(fk_sub) => match fk_sub {
                    FunctionappFunctionKeysCommands::List(args) => {
                        cmd_handlers::wrap(crate::commands::functionapp::function_keys_list(&args.resource_group, &args.name, &args.function_name).await)
                    }
                    FunctionappFunctionKeysCommands::Set(args) => {
                        cmd_handlers::wrap(crate::commands::functionapp::function_keys_set(&args.resource_group, &args.name, &args.function_name, &args.key_name, args.key_value.as_deref()).await)
                    }
                    FunctionappFunctionKeysCommands::Delete(args) => {
                        if !args.yes && !cmd_handlers::confirm(&format!("delete function key '{}'", args.key_name)) {
                            return;
                        }
                        cmd_handlers::wrap_none(crate::commands::functionapp::function_keys_delete(&args.resource_group, &args.name, &args.function_name, &args.key_name).await)
                    }
                },
            },
            FunctionappCommands::Deployment(dep_sub) => match dep_sub {
                FunctionappDeploymentCommands::ListPublishingProfiles(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp::deployment_list_publishing_profiles(&args.resource_group, &args.name).await)
                }
                FunctionappDeploymentCommands::Source(src_sub) => match src_sub {
                    FunctionappDeploymentSourceCommands::ConfigZip(args) => {
                        cmd_handlers::wrap(crate::commands::functionapp::deployment_source_config_zip(&args.resource_group, &args.name, &args.src).await)
                    }
                },
            },
            FunctionappCommands::Plan(plan_sub) => match plan_sub {
                FunctionappPlanCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::functionapp_ext::plan_list(args.resource_group.as_deref()).await)
                }
                FunctionappPlanCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp_ext::plan_show(&args.resource_group, &args.name).await)
                }
                FunctionappPlanCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp_ext::plan_create(
                        &args.resource_group,
                        &args.name,
                        &args.location,
                        args.sku.as_deref(),
                        args.is_linux,
                        args.max_burst,
                        args.tags.as_deref(),
                    ).await)
                }
                FunctionappPlanCommands::Delete(args) => {
                    if !args.yes && !cmd_handlers::confirm(&format!("delete function app plan '{}'", args.name)) {
                        return;
                    }
                    cmd_handlers::wrap_none(crate::commands::functionapp_ext::plan_delete(&args.resource_group, &args.name).await)
                }
                FunctionappPlanCommands::Update(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp_ext::plan_update(
                        &args.resource_group,
                        &args.name,
                        args.sku.as_deref(),
                        args.max_burst,
                        args.number_of_workers,
                    ).await)
                }
            },
            FunctionappCommands::DeploymentSlot(slot_sub) => match slot_sub {
                FunctionappDeploymentSlotCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::functionapp_ext::deployment_slot_list(&args.resource_group, &args.name).await)
                }
                FunctionappDeploymentSlotCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp_ext::deployment_slot_create(&args.resource_group, &args.name, &args.slot).await)
                }
                FunctionappDeploymentSlotCommands::Delete(args) => {
                    if !args.yes && !cmd_handlers::confirm(&format!("delete deployment slot '{}'", args.slot)) {
                        return;
                    }
                    cmd_handlers::wrap_none(crate::commands::functionapp_ext::deployment_slot_delete(&args.resource_group, &args.name, &args.slot).await)
                }
                FunctionappDeploymentSlotCommands::Swap(args) => {
                    cmd_handlers::wrap_none(crate::commands::functionapp_ext::deployment_slot_swap(
                        &args.resource_group,
                        &args.name,
                        &args.slot,
                        args.target_slot.as_deref(),
                    ).await)
                }
            },
            FunctionappCommands::VnetIntegration(vnet_sub) => match vnet_sub {
                FunctionappVnetIntegrationCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::functionapp_ext::vnet_integration_list(&args.resource_group, &args.name).await)
                }
                FunctionappVnetIntegrationCommands::Add(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp_ext::vnet_integration_add(
                        &args.resource_group,
                        &args.name,
                        &args.vnet,
                        &args.subnet,
                    ).await)
                }
                FunctionappVnetIntegrationCommands::Remove(args) => {
                    cmd_handlers::wrap_none(crate::commands::functionapp_ext::vnet_integration_remove(&args.resource_group, &args.name).await)
                }
            },
            FunctionappCommands::ScaleConfig(sc_sub) => match sc_sub {
                FunctionappScaleConfigCommands::Set(args) => {
                    cmd_handlers::wrap(crate::commands::functionapp_ext::scale_config_set(
                        &args.resource_group,
                        &args.name,
                        args.max_burst,
                        args.trigger_type.as_deref(),
                        args.trigger_value.as_deref(),
                    ).await)
                }
            },
        },
        Commands::Appservice(sub) => match sub {
            AppserviceCommands::ListLocations => {
                cmd_handlers::wrap(crate::commands::appservice::list_locations().await)
            }
            AppserviceCommands::Plan(plan_sub) => match plan_sub {
                AppservicePlanCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::appservice::plan_list(args.resource_group.as_deref()).await)
                }
                AppservicePlanCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::appservice::plan_show(&args.resource_group, &args.name).await)
                }
                AppservicePlanCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::appservice::plan_create(
                        &args.resource_group,
                        &args.name,
                        &args.location,
                        args.sku.as_deref(),
                        args.is_linux,
                        args.tags.as_deref(),
                    ).await)
                }
                AppservicePlanCommands::Delete(args) => {
                    if !args.yes && !cmd_handlers::confirm(&format!("delete App Service plan '{}'", args.name)) {
                        return;
                    }
                    cmd_handlers::wrap_none(crate::commands::appservice::plan_delete(&args.resource_group, &args.name).await)
                }
                AppservicePlanCommands::Update(args) => {
                    cmd_handlers::wrap(crate::commands::appservice::plan_update(
                        &args.resource_group,
                        &args.name,
                        args.sku.as_deref(),
                        args.number_of_workers,
                    ).await)
                }
                AppservicePlanCommands::Identity(id_sub) => match id_sub {
                    AppservicePlanIdentityCommands::Assign(args) => {
                        cmd_handlers::wrap(crate::commands::appservice::plan_identity_assign(
                            &args.resource_group,
                            &args.name,
                            args.identity_type.as_deref(),
                        ).await)
                    }
                    AppservicePlanIdentityCommands::Remove(args) => {
                        cmd_handlers::wrap(crate::commands::appservice::plan_identity_remove(
                            &args.resource_group,
                            &args.name,
                        ).await)
                    }
                },
            },
            AppserviceCommands::Ase(ase_sub) => match ase_sub {
                AppserviceAseCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::appservice::ase_list(args.resource_group.as_deref()).await)
                }
                AppserviceAseCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::appservice::ase_show(&args.resource_group, &args.name).await)
                }
                AppserviceAseCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::appservice::ase_create(
                        &args.resource_group,
                        &args.name,
                        &args.location,
                        &args.vnet_name,
                        &args.subnet,
                        args.kind.as_deref(),
                        args.tags.as_deref(),
                    ).await)
                }
                AppserviceAseCommands::Delete(args) => {
                    if !args.yes && !cmd_handlers::confirm(&format!("delete ASE '{}'", args.name)) {
                        return;
                    }
                    cmd_handlers::wrap_none(crate::commands::appservice::ase_delete(&args.resource_group, &args.name).await)
                }
                AppserviceAseCommands::Update(args) => {
                    cmd_handlers::wrap(crate::commands::appservice::ase_update(
                        &args.resource_group,
                        &args.name,
                        args.set.as_deref(),
                    ).await)
                }
                AppserviceAseCommands::ListAddresses(args) => {
                    cmd_handlers::wrap(crate::commands::appservice::ase_list_addresses(&args.resource_group, &args.name).await)
                }
                AppserviceAseCommands::ListPlans(args) => {
                    cmd_handlers::wrap_list(crate::commands::appservice::ase_list_plans(&args.resource_group, &args.name).await)
                }
                AppserviceAseCommands::Upgrade(args) => {
                    if !args.yes && !cmd_handlers::confirm(&format!("upgrade ASE '{}'", args.name)) {
                        return;
                    }
                    cmd_handlers::wrap_none(crate::commands::appservice::ase_upgrade(&args.resource_group, &args.name).await)
                }
            },
            AppserviceCommands::Domain(dom_sub) => match dom_sub {
                AppserviceDomainCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::appservice::domain_create(
                        &args.resource_group,
                        &args.hostname,
                        &args.contact_info,
                    ).await)
                }
                AppserviceDomainCommands::ShowTerms => {
                    cmd_handlers::wrap(crate::commands::appservice::domain_show_terms().await)
                }
            },
        },
        Commands::Staticwebapp(sub) => match sub {
            StaticwebappCommands::List(args) => {
                cmd_handlers::wrap_list(crate::commands::staticwebapp::list(args.resource_group.as_deref()).await)
            }
            StaticwebappCommands::Show(args) => {
                cmd_handlers::wrap(crate::commands::staticwebapp::show(&args.resource_group, &args.name).await)
            }
            StaticwebappCommands::Create(args) => {
                cmd_handlers::wrap(crate::commands::staticwebapp::create(
                    &args.resource_group,
                    &args.name,
                    &args.location,
                    args.source.as_deref(),
                    args.branch.as_deref(),
                    args.token.as_deref(),
                    args.sku.as_deref(),
                    args.tags.as_deref(),
                ).await)
            }
            StaticwebappCommands::Delete(args) => {
                if !args.yes && !cmd_handlers::confirm(&format!("delete static web app '{}'", args.name)) {
                    return;
                }
                cmd_handlers::wrap_none(crate::commands::staticwebapp::delete(&args.resource_group, &args.name).await)
            }
            StaticwebappCommands::Update(args) => {
                cmd_handlers::wrap(crate::commands::staticwebapp::update(&args.resource_group, &args.name, args.set.as_deref()).await)
            }
            StaticwebappCommands::Appsettings(as_sub) => match as_sub {
                StaticwebappAppsettingsCommands::List(args) => {
                    cmd_handlers::wrap(crate::commands::staticwebapp::appsettings_list(&args.resource_group, &args.name).await)
                }
                StaticwebappAppsettingsCommands::Set(args) => {
                    cmd_handlers::wrap(crate::commands::staticwebapp::appsettings_set(&args.resource_group, &args.name, &args.settings).await)
                }
                StaticwebappAppsettingsCommands::Delete(args) => {
                    cmd_handlers::wrap(crate::commands::staticwebapp::appsettings_delete(&args.resource_group, &args.name, &args.setting_names).await)
                }
            },
            StaticwebappCommands::Hostname(hn_sub) => match hn_sub {
                StaticwebappHostnameCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::staticwebapp::hostname_list(&args.resource_group, &args.name).await)
                }
                StaticwebappHostnameCommands::Set(args) => {
                    cmd_handlers::wrap(crate::commands::staticwebapp::hostname_set(&args.resource_group, &args.name, &args.hostname).await)
                }
                StaticwebappHostnameCommands::Delete(args) => {
                    cmd_handlers::wrap_none(crate::commands::staticwebapp::hostname_delete(&args.resource_group, &args.name, &args.hostname).await)
                }
            },
            StaticwebappCommands::Environment(env_sub) => match env_sub {
                StaticwebappEnvironmentCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::staticwebapp::environment_list(&args.resource_group, &args.name).await)
                }
                StaticwebappEnvironmentCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::staticwebapp::environment_show(&args.resource_group, &args.name, &args.environment_name).await)
                }
                StaticwebappEnvironmentCommands::Delete(args) => {
                    cmd_handlers::wrap_none(crate::commands::staticwebapp::environment_delete(&args.resource_group, &args.name, &args.environment_name).await)
                }
            },
        },
        Commands::Storage(sub) => match sub {
            StorageCommands::Account(acct) => match acct {
                StorageAccountCommands::Create(args) => {
                    cmd_handlers::wrap(crate::commands::storage::create(&args.name, &args.resource_group, &args.location, &args.sku, &args.kind).await)
                }
                StorageAccountCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::storage::list(args.resource_group.as_deref()).await)
                }
                StorageAccountCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::storage::show(&args.name, &args.resource_group).await)
                }
                StorageAccountCommands::Delete(args) => {
                    if !args.yes && !cmd_handlers::confirm(&format!("delete storage account '{}'", args.name)) {
                        return;
                    }
                    cmd_handlers::wrap_none(crate::commands::storage::delete(&args.name, &args.resource_group).await)
                }
                StorageAccountCommands::Keys(args) => {
                    cmd_handlers::wrap(crate::commands::storage::keys_list(&args.name, &args.resource_group).await)
                }
            },
        },
        Commands::Config(sub) => match sub {
            ConfigCommands::Set(args) => {
                crate::config::config_set(&args.pairs).map(|_| None)
            }
            ConfigCommands::Get(args) => {
                crate::config::config_get(&args.key)
            }
            ConfigCommands::Unset(args) => {
                crate::config::config_unset(&args.key).map(|_| None)
            }
        },
        Commands::Keyvault(sub) => match sub {
            KeyvaultCommands::Secret(scmd) => match scmd {
                KeyvaultSecretCommands::Set(args) => {
                    cmd_handlers::wrap(crate::commands::keyvault::secret_set(&args.vault_name, &args.name, &args.value).await)
                }
                KeyvaultSecretCommands::Show(args) => {
                    cmd_handlers::wrap(crate::commands::keyvault::secret_show(&args.vault_name, &args.name).await)
                }
                KeyvaultSecretCommands::List(args) => {
                    cmd_handlers::wrap_list(crate::commands::keyvault::secret_list(&args.vault_name).await)
                }
                KeyvaultSecretCommands::Delete(args) => {
                    cmd_handlers::wrap(crate::commands::keyvault::secret_delete(&args.vault_name, &args.name).await)
                }
            },
        },
        Commands::Generated(sub) => {
            crate::generated::dispatch_generated(sub).await
        },
    };

    match result {
        Ok(Some(value)) => {
            if let Err(e) = output::format_and_print(&value, output_format, query.as_deref()) {
                eprintln!("ERROR: {e}");
                std::process::exit(1);
            }
        }
        Ok(None) => {} // No output (e.g. delete, set, login, logout)
        Err(e) => {
            eprintln!("ERROR: {e}");
            if let Some(suggestion) = e.suggestion() {
                eprintln!(
                    "Run the command below to authenticate interactively; \
                     additional arguments may be added as needed:"
                );
                eprintln!("{suggestion}");
            }
            std::process::exit(1);
        }
    }
}

async fn dispatch_deployment_scope(
    sub: DeploymentScopeCommands,
    scope: crate::commands::deployment::Scope<'_>,
) -> CmdResult {
    use crate::commands::deployment;
    match sub {
        DeploymentScopeCommands::List(args) => {
            cmd_handlers::wrap_list(deployment::list(scope, args.filter.as_deref()).await)
        }
        DeploymentScopeCommands::Show(args) => {
            cmd_handlers::wrap(deployment::show(scope, &args.name).await)
        }
        DeploymentScopeCommands::Delete(args) => {
            cmd_handlers::wrap_none(deployment::delete(scope, &args.name).await)
        }
        DeploymentScopeCommands::Create(args) => {
            cmd_handlers::wrap(deployment::create(
                scope,
                &args.name,
                args.template_file.as_deref(),
                args.template_uri.as_deref(),
                args.parameters.as_deref(),
                args.mode.as_deref(),
                args.no_wait,
            ).await)
        }
        DeploymentScopeCommands::Validate(args) => {
            cmd_handlers::wrap(deployment::validate(
                scope,
                &args.name,
                args.template_file.as_deref(),
                args.template_uri.as_deref(),
                args.parameters.as_deref(),
            ).await)
        }
        DeploymentScopeCommands::Export(args) => {
            cmd_handlers::wrap(deployment::export(scope, &args.name).await)
        }
        DeploymentScopeCommands::Cancel(args) => {
            cmd_handlers::wrap_none(deployment::cancel(scope, &args.name).await)
        }
        DeploymentScopeCommands::WhatIf(args) => {
            cmd_handlers::wrap(deployment::what_if(
                scope,
                &args.name,
                args.template_file.as_deref(),
                args.template_uri.as_deref(),
                args.parameters.as_deref(),
            ).await)
        }
        DeploymentScopeCommands::Operation(op) => match op {
            DeploymentOperationCommands::List(args) => {
                cmd_handlers::wrap_list(deployment::operation_list(scope, &args.name).await)
            }
        },
    }
}

async fn dispatch_stack_scope(
    sub: StackScopeCommands,
    scope: crate::commands::stack::StackScope<'_>,
) -> CmdResult {
    use crate::commands::stack;
    match sub {
        StackScopeCommands::List(_args) => {
            cmd_handlers::wrap_list(stack::list(scope).await)
        }
        StackScopeCommands::Show(args) => {
            cmd_handlers::wrap(stack::show(scope, &args.name).await)
        }
        StackScopeCommands::Delete(args) => {
            cmd_handlers::wrap_none(stack::delete(scope, &args.name).await)
        }
        StackScopeCommands::Export(args) => {
            cmd_handlers::wrap(stack::export(scope, &args.name).await)
        }
    }
}

mod cmd_handlers {
    use crate::auth::TokenCache;
    use crate::cli::*;
    use crate::cloud::CloudConfig;
    use crate::error::Result;
    use crate::profile::Profile;
    use crate::arm::ArmClient;
    use crate::output;

    type CmdResult = Result<Option<serde_json::Value>>;

    /// Wrap a Result<Value> into CmdResult.
    pub fn wrap(r: Result<serde_json::Value>) -> CmdResult {
        r.map(Some)
    }

    /// Wrap a Result<Vec<Value>> into CmdResult (as JSON array).
    pub fn wrap_list(r: Result<Vec<serde_json::Value>>) -> CmdResult {
        r.map(|v| Some(serde_json::Value::Array(v)))
    }

    /// Wrap a Result<()> into CmdResult (no output).
    pub fn wrap_none(r: Result<()>) -> CmdResult {
        r.map(|_| None)
    }

    /// Interactive confirmation prompt. Returns true if user confirms.
    pub fn confirm(action: &str) -> bool {
        use std::io::{self, BufRead, Write};
        eprint!("Are you sure you want to {action}? (y/N): ");
        io::stderr().flush().ok();
        let line = io::stdin().lock().lines().next();
        matches!(line, Some(Ok(input)) if input.trim().eq_ignore_ascii_case("y"))
    }

    pub async fn login(args: LoginArgs) -> CmdResult {
        let cloud = CloudConfig::default();
        let authority_host = &cloud.active_directory;

        if args.service_principal {
            // Service principal login
            let client_id = args.username.as_deref().ok_or_else(|| {
                crate::error::AzrsError::General(
                    "Service principal login requires --username/-u (client/app ID).".into(),
                )
            })?;
            let client_secret = args.password.as_deref().ok_or_else(|| {
                crate::error::AzrsError::General(
                    "Service principal login requires --password/-p (client secret).".into(),
                )
            })?;
            let tenant = args.tenant.as_deref().ok_or_else(|| {
                crate::error::AzrsError::General(
                    "Service principal login requires --tenant/-t.".into(),
                )
            })?;

            let authority = format!("{authority_host}/{tenant}");
            let scopes = args
                .scope
                .unwrap_or_else(|| vec![cloud.default_scope()]);

            eprintln!("Logging in as service principal...");
            let token_response = crate::auth::service_principal::login_with_secret(
                &authority, client_id, client_secret, &scopes,
            )
            .await?;

            // Store tokens
            let mut cache = TokenCache::load(&cloud)?;
            cache.store_tokens_for(&token_response, client_id, tenant)?;

            // Store SP entry for future token refresh
            let mut sp_store = crate::auth::service_principal::SpStore::load();
            sp_store.upsert(crate::auth::service_principal::SpEntry {
                client_id: client_id.to_string(),
                tenant: tenant.to_string(),
                client_secret: Some(client_secret.to_string()),
                certificate: None,
            });
            sp_store.save()?;

            // Discover subscriptions
            eprintln!("Retrieving subscriptions...");
            let arm = ArmClient::new(&cloud);
            let subscriptions = arm
                .discover_subscriptions_for_tenant(tenant, &token_response.access_token)
                .await?;

            if subscriptions.is_empty() && !args.allow_no_subscriptions {
                eprintln!("No subscriptions found. Use --allow-no-subscriptions to log in without one.");
            }

            // Build subscription entries with SP user type
            let sp_subs: Vec<crate::profile::Subscription> = subscriptions
                .into_iter()
                .map(|mut s| {
                    s.user = crate::profile::SubscriptionUser {
                        name: client_id.to_string(),
                        user_type: "servicePrincipal".to_string(),
                    };
                    s
                })
                .collect();

            let mut profile = Profile::load()?;
            profile.merge_subscriptions(sp_subs, &None, &cloud.environment_name);
            profile.save()?;
            cache.save()?;

            output::print_login_summary(&profile);
            return Ok(None);
        }

        // Interactive / device code login
        let tenant = args.tenant.as_deref().unwrap_or("organizations");
        let authority = format!("{authority_host}/{tenant}");
        let scopes = args.scope.unwrap_or_else(|| vec![cloud.default_scope()]);

        eprintln!("Opening login...");
        let token_response = if args.use_device_code {
            crate::auth::device_code::login(&authority, &scopes).await?
        } else {
            crate::auth::interactive::login(&authority, &scopes).await?
        };

        let mut cache = TokenCache::load(&cloud)?;
        cache.store_tokens(&token_response)?;

        eprintln!("Retrieving tenants and subscriptions for the selection...");
        let arm = ArmClient::new(&cloud);
        let subscriptions = if args.tenant.is_some() {
            arm.discover_subscriptions_for_tenant(
                tenant,
                &token_response.access_token,
            )
            .await?
        } else {
            arm.discover_all_subscriptions(&token_response, &mut cache)
                .await?
        };

        if subscriptions.is_empty() && !args.allow_no_subscriptions {
            eprintln!(
                "No subscriptions found. Use --allow-no-subscriptions to log in without one."
            );
        }

        let mut profile = Profile::load()?;

        if !subscriptions.is_empty() && crate::selector::is_interactive() {
            if let Some(selected_idx) = crate::selector::select_subscription(&subscriptions) {
                let mut subs_with_default = subscriptions;
                for (i, sub) in subs_with_default.iter_mut().enumerate() {
                    sub.is_default = i == selected_idx;
                }
                profile.merge_subscriptions(
                    subs_with_default,
                    &token_response.id_token_claims,
                    &cloud.environment_name,
                );
            }
        } else {
            profile.merge_subscriptions(
                subscriptions,
                &token_response.id_token_claims,
                &cloud.environment_name,
            );
        }

        profile.save()?;
        cache.save()?;

        output::print_login_summary(&profile);
        Ok(None) // Login output is on stderr, not formatted
    }

    pub async fn logout(args: LogoutArgs) -> CmdResult {
        let cloud = CloudConfig::default();
        let mut profile = Profile::load()?;
        let mut cache = TokenCache::load(&cloud)?;

        let username = match args.username {
            Some(u) => u,
            None => profile
                .active_subscription()
                .map(|s| s.user.name.clone())
                .ok_or_else(|| {
                    crate::error::AzrsError::General(
                        "No active account. Specify --username.".into(),
                    )
                })?,
        };

        profile.remove_subscriptions_for_user(&username);
        cache.remove_tokens_for_user(&username);

        profile.save()?;
        cache.save()?;

        eprintln!("Logged out: {username}");
        Ok(None)
    }

    pub async fn account_show() -> CmdResult {
        let profile = Profile::load()?;
        let sub = profile
            .active_subscription()
            .ok_or(crate::error::AzrsError::NoActiveSubscription)?;
        Ok(Some(serde_json::to_value(sub)?))
    }

    pub async fn account_list(_args: AccountListArgs) -> CmdResult {
        let profile = Profile::load()?;
        Ok(Some(serde_json::to_value(&profile.subscriptions)?))
    }

    pub async fn account_set(args: AccountSetArgs) -> CmdResult {
        let mut profile = Profile::load()?;
        profile.set_active_subscription(&args.subscription)?;
        profile.save()?;
        Ok(None)
    }

    pub async fn account_get_access_token(args: GetAccessTokenArgs) -> CmdResult {
        let cloud = CloudConfig::default();
        let profile = Profile::load()?;
        let mut cache = TokenCache::load(&cloud)?;

        let sub = if let Some(ref sub_id) = args.subscription {
            profile
                .find_subscription(sub_id)
                .ok_or_else(|| crate::error::AzrsError::SubscriptionNotFound(sub_id.clone()))?
        } else {
            profile
                .active_subscription()
                .ok_or(crate::error::AzrsError::NoActiveSubscription)?
        };

        let scopes = if let Some(scopes) = args.scope {
            scopes
        } else if let Some(resource) = args.resource {
            vec![crate::cloud::resource_to_scope(&resource)]
        } else {
            vec![cloud.default_scope()]
        };

        let tenant = args.tenant.as_deref().unwrap_or(&sub.tenant_id);
        let token = cache.get_access_token(&sub.user.name, tenant, &scopes, &cloud).await?;

        let result = serde_json::json!({
            "accessToken": token.access_token,
            "expiresOn": token.expires_on.format("%Y-%m-%d %H:%M:%S").to_string(),
            "subscription": sub.id,
            "tenant": tenant,
            "tokenType": "Bearer",
        });

        Ok(Some(result))
    }

    pub async fn rest(args: RestArgs) -> CmdResult {
        crate::rest::send_raw_request(
            &args.method,
            &args.url,
            args.headers.as_deref(),
            args.uri_parameters.as_deref(),
            args.body.as_deref(),
            args.skip_authorization_header,
            args.resource.as_deref(),
            args.output_file.as_deref(),
        )
        .await?;
        Ok(None) // rest handles its own output
    }

    pub async fn list_locations() -> Result<serde_json::Value> {
        let mut cmd = crate::commands::ArmCommand::new()?;
        let path = "/subscriptions/{subscriptionId}/locations?api-version=2024-03-01";
        let result = cmd.list(path).await?;
        cmd.save_cache()?;
        Ok(serde_json::Value::Array(result))
    }
}
