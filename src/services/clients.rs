use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    Set, TransactionTrait,
};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    dtos::{
        client::{
            ClientAuthMethod, ClientAuthStatusDto, ClientMagicLinkDto, ClientSearchField, ClientSummary,
            CreateClientDto, SaveClientCredentialsDto, UpdateClientDto, UpdateClientStatusDto,
        },
        query::QueryPayload,
    },
    entities::{
        client_auth,
        clients::{self, Model as ClientModel},
        prelude::{ClientAuth, Clients},
    },
    error::AppError,
    utils::{
        crypto::{generate_magic_link_token, hash_password},
        search::{apply_filters, apply_ordering, apply_pagination},
    },
};

pub struct ClientService;

impl ClientService {
    pub async fn create(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        dto: CreateClientDto,
    ) -> Result<ClientModel, AppError> {
        let txn = db.begin().await.map_err(AppError::Database)?;

        let client = clients::ActiveModel {
            name: Set(dto.name),
            user_id: Set(user_id),
            email: Set(dto.email),
            phone: Set(dto.phone),
            address_line: Set(dto.address_line),
            company: Set(dto.company),
            notes: Set(dto.notes),
            ..Default::default()
        };

        let created_client = client.insert(&txn).await.map_err(AppError::Database)?;

        txn.commit().await.map_err(AppError::Database)?;

        Ok(created_client)
    }

    pub async fn get_by_id(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<ClientModel, AppError> {
        let client = Clients::find()
            .filter(clients::Column::Id.eq(id))
            .filter(clients::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("client not found".to_string()))?;

        Ok(client)
    }

    pub async fn update(
        db: &(impl ConnectionTrait + TransactionTrait),
        user_id: Uuid,
        id: Uuid,
        dto: UpdateClientDto,
    ) -> Result<ClientModel, AppError> {
        let client = Self::get_by_id(db, user_id, id).await?;
        let txn = db.begin().await.map_err(AppError::Database)?;

        let mut active_client: clients::ActiveModel = client.into();
        active_client.name = Set(dto.name);
        active_client.status = Set(dto.status.to_string());
        active_client.email = Set(dto.email);
        active_client.phone = Set(dto.phone);
        active_client.address_line = Set(dto.address_line);
        active_client.company = Set(dto.company);
        active_client.notes = Set(dto.notes);

        let updated_client = active_client.update(&txn).await.map_err(AppError::Database)?;

        txn.commit().await.map_err(AppError::Database)?;

        Ok(updated_client)
    }

    pub async fn update_client_status(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateClientStatusDto,
    ) -> Result<ClientModel, AppError> {
        let client = Self::get_by_id(db, user_id, id).await?;
        let mut active_client: clients::ActiveModel = client.into();
        active_client.status = Set(dto.status.to_string());

        active_client.update(db).await.map_err(AppError::Database)
    }

    pub async fn delete(db: &impl ConnectionTrait, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let client = Self::get_by_id(db, user_id, id).await?;

        clients::ActiveModel {
            id: Set(client.id),
            ..Default::default()
        }
        .delete(db)
        .await?;

        Ok(())
    }

    pub async fn search(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        payload: QueryPayload,
    ) -> Result<(Vec<ClientSummary>, u64), AppError> {
        let base_condition = Condition::all().add(clients::Column::UserId.eq(user_id));
        let condition = apply_filters(base_condition, &payload.filters, |column| {
            column
                .parse::<ClientSearchField>()
                .ok()
                .map(ClientSearchField::filter_field)
        })?;

        let query = Clients::find().filter(condition);
        let query = apply_ordering(
            query,
            &payload,
            ClientSearchField::CreatedAt.order_column(),
            false,
            |column| {
                column
                    .parse::<ClientSearchField>()
                    .ok()
                    .map(ClientSearchField::order_column)
            },
        )?;

        let total_items = query.clone().count(db).await?;

        let models = apply_pagination(
            query
                .select_only()
                .column(clients::Column::Id)
                .column(clients::Column::Name)
                .column(clients::Column::Status)
                .column(clients::Column::CreatedAt)
                .column(clients::Column::UpdatedAt),
            &payload,
        )
        .into_model::<ClientSummary>()
        .all(db)
        .await?;

        Ok((models, total_items))
    }

    pub async fn generate_client_magic_token(
        db: &impl ConnectionTrait,
        magic_token_ttl: Duration,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<ClientMagicLinkDto, AppError> {
        Self::get_by_id(db, user_id, client_id).await?;

        let (token, hashed_token) = generate_magic_link_token();
        let time = Utc::now();
        let ttl_seconds = i64::try_from(magic_token_ttl.as_secs())
            .map_err(|_| AppError::Internal("magic token ttl out of range".to_string()))?;
        let expiration_time = time
            .checked_add_signed(chrono::Duration::seconds(ttl_seconds))
            .ok_or_else(|| AppError::Internal("could not compute magic token expiration".to_string()))?;
        let timestamp = expiration_time.timestamp();
        if let Some(offset) = FixedOffset::east_opt(0) {
            let fixed_time: DateTime<FixedOffset> = expiration_time.with_timezone(&offset);
            let mut is_insertion: bool = false;
            let mut active_client_auth: client_auth::ActiveModel =
                match Self::get_client_auth(db, user_id, client_id).await {
                    Ok(active_client_auth) => active_client_auth.into(),
                    Err(err) => match err {
                        AppError::NotFound(_) => {
                            is_insertion = true;
                            client_auth::ActiveModel {
                                client_id: Set(client_id),
                                user_id: Set(user_id),
                                ..Default::default()
                            }
                        }
                        _ => return Err(err),
                    },
                };

            active_client_auth.magic_link_hash = Set(Some(hashed_token));
            active_client_auth.magic_link_exp = Set(Some(fixed_time));
            active_client_auth.email = Set(None);
            active_client_auth.password_hash = Set(None);

            let _ = if is_insertion {
                active_client_auth.insert(db).await?
            } else {
                active_client_auth.update(db).await?
            };

            Ok(ClientMagicLinkDto {
                magic_client_token: token,
                magic_client_token_exp: timestamp,
            })
        } else {
            Err(AppError::Internal("offset error".to_string()))
        }
    }

    pub async fn save_client_credentials(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
        dto: SaveClientCredentialsDto,
    ) -> Result<(), AppError> {
        Self::get_by_id(db, user_id, client_id).await?;

        let mut is_insertion: bool = false;
        let mut client_auth: client_auth::ActiveModel = match Self::get_client_auth(db, user_id, client_id).await {
            Ok(client_auth) => client_auth.into(),
            Err(err) => match err {
                AppError::NotFound(_) => {
                    is_insertion = true;
                    client_auth::ActiveModel {
                        user_id: Set(user_id),
                        client_id: Set(client_id),
                        ..Default::default()
                    }
                }
                _ => return Err(err),
            },
        };

        let client_password_hash = hash_password(dto.password)?;

        client_auth.email = Set(Some(dto.email));
        client_auth.password_hash = Set(Some(client_password_hash));
        client_auth.magic_link_hash = Set(None);
        client_auth.magic_link_exp = Set(None);

        let _ = if is_insertion {
            client_auth.insert(db).await?
        } else {
            client_auth.update(db).await?
        };

        Ok(())
    }

    pub async fn get_client_auth_status(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<ClientAuthStatusDto, AppError> {
        Self::get_by_id(db, user_id, client_id).await?;

        let client_auth = match Self::get_client_auth(db, user_id, client_id).await {
            Ok(model) => model,
            Err(AppError::NotFound(_)) => {
                return Ok(ClientAuthStatusDto {
                    method: ClientAuthMethod::None,
                    email: None,
                    magic_client_token_exp: None,
                });
            }
            Err(err) => return Err(err),
        };

        let has_credentials = client_auth.email.is_some() && client_auth.password_hash.is_some();
        let has_magic_token = client_auth.magic_link_hash.is_some() && client_auth.magic_link_exp.is_some();

        if has_credentials && has_magic_token {
            return Err(AppError::Internal("invalid client authentication state".to_string()));
        }

        if has_credentials {
            return Ok(ClientAuthStatusDto {
                method: ClientAuthMethod::Credentials,
                email: client_auth.email,
                magic_client_token_exp: None,
            });
        }

        if has_magic_token {
            return Ok(ClientAuthStatusDto {
                method: ClientAuthMethod::MagicToken,
                email: None,
                magic_client_token_exp: client_auth.magic_link_exp.map(|date| date.timestamp()),
            });
        }

        Ok(ClientAuthStatusDto {
            method: ClientAuthMethod::None,
            email: None,
            magic_client_token_exp: None,
        })
    }

    async fn get_client_auth(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<client_auth::Model, AppError> {
        let client_auth = ClientAuth::find()
            .filter(client_auth::Column::UserId.eq(user_id))
            .filter(client_auth::Column::ClientId.eq(client_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::NotFound("client authentication data not found".to_string()))?;

        Ok(client_auth)
    }
}
