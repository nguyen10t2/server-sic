use crate::database::schema::Payload;

pub struct PayloadRepository {
    pool: sqlx::PgPool,
}

impl PayloadRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl PayloadRepository {
    pub async fn save_payload(&self, payload: &Payload) -> anyhow::Result<()> {
        sqlx::query(
            r#"
                INSERT INTO payloads (
                    timestamp,
                    temperature,
                    humidity,
                    smoke,
                    flame,
                    node_id,
                    battery,
                    status
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(payload.timestamp)
        .bind(payload.temperature)
        .bind(payload.humidity)
        .bind(payload.smoke)
        .bind(payload.flame)
        .bind(payload.node_id as i32)
        .bind(payload.battery as i32)
        .bind(payload.status as i32) // Chuyển enum thành u8 để lưu vào database
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn save_payloads_batch(&self, payloads: &[Payload]) -> anyhow::Result<()> {
        if payloads.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO payloads (timestamp, temperature, humidity, smoke, flame, node_id, battery, status) ",
        );

        query_builder.push_values(payloads, |mut b, payload| {
            b.push_bind(payload.timestamp)
                .push_bind(payload.temperature)
                .push_bind(payload.humidity)
                .push_bind(payload.smoke)
                .push_bind(payload.flame)
                .push_bind(payload.node_id as i32)
                .push_bind(payload.battery as i32)
                .push_bind(payload.status as i32);
        });

        let query = query_builder.build();
        query.execute(&self.pool).await?;

        Ok(())
    }
}
