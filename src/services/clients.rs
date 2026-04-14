use axum::extract::FromRequestParts;
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        client::{
            ClientMagicLinkDto, ClientSummary, CreateClientDto, SaveClientCredentialsDto, UpdateClientDto,
            UpdateClientStatusDto,
        },
        query::{FilterOperator, QueryPayload},
    },
    entities::{
        client_auth,
        clients::{self, Model as ClientModel},
        prelude::{ClientAuth, Clients},
    },
    error::AppError,
    state::AppState,
    utils::crypto::{generate_magic_link_token, hash_password},
};

#[derive(Clone)]
pub struct ClientService {
    pub db: Arc<DatabaseConnection>,
}

impl ClientService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    pub async fn create(&self, user_id: Uuid, dto: CreateClientDto) -> Result<ClientModel, AppError> {
        let txn = self.db.begin().await.map_err(AppError::Database)?;

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

    pub async fn get_by_id(&self, user_id: Uuid, id: Uuid) -> Result<ClientModel, AppError> {
        let client = Clients::find()
            .filter(clients::Column::Id.eq(id))
            .filter(clients::Column::UserId.eq(user_id))
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Cliente no encontrado".to_string()))?;

        Ok(client)
    }

    pub async fn update(&self, user_id: Uuid, id: Uuid, dto: UpdateClientDto) -> Result<ClientModel, AppError> {
        let client = self.get_by_id(user_id, id).await?;
        let txn = self.db.begin().await.map_err(AppError::Database)?;

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
        &self,
        user_id: Uuid,
        id: Uuid,
        dto: UpdateClientStatusDto,
    ) -> Result<ClientModel, AppError> {
        let client = self.get_by_id(user_id, id).await?;
        let mut active_client: clients::ActiveModel = client.into();
        active_client.status = Set(dto.status.to_string());

        active_client.update(self.db.as_ref()).await.map_err(AppError::Database)
    }

    pub async fn delete(&self, user_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let client = self.get_by_id(user_id, id).await?;

        clients::ActiveModel {
            id: Set(client.id),
            ..Default::default()
        }
        .delete(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn search(&self, user_id: Uuid, payload: QueryPayload) -> Result<(Vec<ClientSummary>, u64), AppError> {
        let mut condition = Condition::all().add(clients::Column::UserId.eq(user_id));

        for filter in payload.filters {
            let filter_cond = match filter.column.as_str() {
                "id" => {
                    let value_str = filter.value.as_str().unwrap_or("");
                    if let Ok(uuid_val) = uuid::Uuid::parse_str(value_str) {
                        let col = clients::Column::Id;
                        match filter.operator {
                            FilterOperator::Eq => col.eq(uuid_val),
                            FilterOperator::Neq => col.ne(uuid_val),
                            _ => continue,
                        }
                    } else {
                        continue;
                    }
                }
                "status" => {
                    let value_str = filter.value.as_str().unwrap_or("");
                    let col = clients::Column::Status;
                    match filter.operator {
                        FilterOperator::Eq => col.eq(value_str),
                        FilterOperator::Neq => col.ne(value_str),
                        _ => continue,
                    }
                }
                "name" => {
                    let col = clients::Column::Name;
                    let val_str = filter.value.as_str().unwrap_or("").to_string();

                    match filter.operator {
                        FilterOperator::Eq => col.eq(val_str),
                        FilterOperator::Neq => col.ne(val_str),
                        FilterOperator::Contains => col.contains(&val_str),
                        FilterOperator::NotContains => col.not_like(&format!("%{}%", val_str)),
                        _ => continue,
                    }
                }
                _ => continue,
            };

            condition = condition.add(filter_cond);
        }

        let mut query = Clients::find().filter(condition);

        if let Some(order_col) = payload.order_by {
            let col = match order_col.as_str() {
                "id" => clients::Column::Id,
                "name" => clients::Column::Name,
                "created_at" => clients::Column::CreatedAt,
                "updated_at" => clients::Column::UpdatedAt,
                _ => clients::Column::CreatedAt,
            };

            let order = if payload.order_asc {
                sea_orm::Order::Asc
            } else {
                sea_orm::Order::Desc
            };

            query = query.order_by(col, order);
        } else {
            query = query.order_by_desc(clients::Column::CreatedAt);
        }

        let total_items = query.clone().count(self.db.as_ref()).await?;
        let offset = (payload.page - 1) * payload.limit;

        let models = query
            .select_only()
            .column(clients::Column::Id)
            .column(clients::Column::Name)
            .column(clients::Column::Status)
            .column(clients::Column::CreatedAt)
            .column(clients::Column::UpdatedAt)
            .limit(payload.limit)
            .offset(offset)
            .into_model::<ClientSummary>()
            .all(self.db.as_ref())
            .await?;

        Ok((models, total_items))
    }

    pub async fn generate_client_magic_token(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<ClientMagicLinkDto, AppError> {
        let (token, hashed_token) = generate_magic_link_token();
        let time = Utc::now();
        let timestamp = time.timestamp();
        if let Some(offset) = FixedOffset::east_opt(0) {
            let fixed_time: DateTime<FixedOffset> = time.with_timezone(&offset);
            let mut is_insertion: bool = false;
            let mut active_client_auth: client_auth::ActiveModel =
                match self.get_client_auth(user_id.clone(), client_id.clone()).await {
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
                active_client_auth.insert(self.db.as_ref()).await?
            } else {
                active_client_auth.update(self.db.as_ref()).await?
            };

            Ok(ClientMagicLinkDto {
                magic_client_token: token,
                magic_client_token_exp: timestamp,
            })
        } else {
            Err(AppError::Internal("Error del offset".to_string()))
        }
    }

    pub async fn save_client_credentials(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        dto: SaveClientCredentialsDto,
    ) -> Result<(), AppError> {
        let mut is_insertion: bool = false;
        let mut client_auth: client_auth::ActiveModel =
            match self.get_client_auth(user_id.clone(), client_id.clone()).await {
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
            client_auth.insert(self.db.as_ref()).await?
        } else {
            client_auth.update(self.db.as_ref()).await?
        };

        Ok(())
    }

    async fn get_client_auth(&self, user_id: Uuid, client_id: Uuid) -> Result<client_auth::Model, AppError> {
        let client_auth = ClientAuth::find()
            .filter(client_auth::Column::UserId.eq(user_id))
            .filter(client_auth::Column::ClientId.eq(client_id))
            .one(self.db.as_ref())
            .await?
            .ok_or_else(|| AppError::NotFound("Datos de autenticación del cliente no encontrados".to_string()))?;

        Ok(client_auth)
    }
}

impl FromRequestParts<AppState> for ClientService {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(ClientService::new(state.db.clone()))
    }
}
