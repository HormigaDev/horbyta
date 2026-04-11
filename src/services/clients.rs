use axum::extract::FromRequestParts;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dtos::{
        client::{ClientSummary, CreateClientDto, UpdateClientDto, UpdateClientStatusDto},
        query::{FilterOperator, QueryPayload},
    },
    entities::{
        clients::{self, Model as ClientModel},
        prelude::Clients,
    },
    error::AppError,
    state::AppState,
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
        active_client.status = Set(dto.status);
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
        active_client.status = Set(dto.status);

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
