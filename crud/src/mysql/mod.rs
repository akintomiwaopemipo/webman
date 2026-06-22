use indexmap::{indexmap, IndexMap};
use sea_query::{
    ColumnRef,
    Cond,
    Condition,
    DeleteStatement,
    Expr,
    Iden,
    IntoIden,
    InsertStatement,
    JoinType,
    MysqlQueryBuilder,
    Order,
    Query,
    SelectStatement,
    SimpleExpr,
    TableRef,
    UpdateStatement,
};
use sea_query_binder::SqlxBinder;
use sqlx::{mysql::MySqlRow, MySql};
use tracing::info;
use utils::{random_hex, substrings::Substrings};

//
// ============================================================
// SINGLE TABLE CRUD
// ============================================================
//

#[derive(Clone)]
pub struct Crud<T>
where
    T: Iden + Clone,
{
    pub table: T,

    pub select: Vec<T>,
    pub select_exprs: Vec<SimpleExpr>,

    pub columns: IndexMap<T, SimpleExpr>,
    pub updates: IndexMap<T, SimpleExpr>,

    expressions: Vec<SimpleExpr>,

    group_by: Vec<T>,
    order_by: Vec<(T, Order)>,

    limit: Option<u64>,

    pool: sqlx::pool::Pool<MySql>,
}

impl<T> Crud<T>
where
    T: Iden
        + Clone
        + 'static
        + core::hash::Hash
        + Eq
        + PartialEq,
{
    pub fn new(
        table: T,
        pool: &sqlx::pool::Pool<MySql>,
    ) -> Self {
        Self {
            table,

            select: vec![],
            select_exprs: vec![],

            columns: indexmap! {},
            updates: indexmap! {},

            expressions: vec![],

            group_by: vec![],
            order_by: vec![],

            limit: None,

            pool: pool.clone(),
        }
    }

    // =========================================
    // SELECT
    // =========================================

    pub fn select_column(
        &mut self,
        column: T,
    ) -> &mut Self {
        self.select.push(column);

        self
    }

    pub fn select_columns<S>(
        &mut self,
        columns: S,
    ) -> &mut Self
    where
        S: IntoIterator<Item = T>,
    {
        self.select =
            columns.into_iter().collect();

        self
    }

    pub fn select_expr(
        &mut self,
        expr: SimpleExpr,
    ) -> &mut Self {
        self.select_exprs.push(expr);

        self
    }

    // =========================================
    // WHERE
    // =========================================

    pub fn set_column(
        &mut self,
        column: T,
        value: SimpleExpr,
    ) -> &mut Self {
        self.columns.insert(
            column,
            value,
        );

        self
    }

    pub fn unset_column(
        &mut self,
        column: T,
    ) -> &mut Self {
        self.columns.shift_remove(
            &column,
        );

        self
    }


    // =========================================
    // SET COLUMNS
    // =========================================

    pub fn set_columns(
        &mut self,
        columns: IndexMap<T, SimpleExpr>,
    ) -> &mut Self {
        self.columns = columns;

        self
    }

    pub fn merge_columns(
        &mut self,
        columns: IndexMap<T, SimpleExpr>,
    ) -> &mut Self {
        for (column, value) in columns {
            self.columns.insert(
                column,
                value,
            );
        }

        self
    }

    // =========================================
    // UPDATE VALUES
    // =========================================

    pub fn update_column(
        &mut self,
        column: T,
        value: SimpleExpr,
    ) -> &mut Self {
        self.updates.insert(
            column,
            value,
        );

        self
    }


    // =========================================
    // SET UPDATE
    // =========================================

    pub fn set_update(
        &mut self,
        column: T,
        value: SimpleExpr,
    ) -> &mut Self {
        self.updates.insert(
            column,
            value,
        );

        self
    }

    // =========================================
    // SET UPDATES
    // =========================================

    pub fn set_updates(
        &mut self,
        updates: IndexMap<T, SimpleExpr>,
    ) -> &mut Self {
        self.updates = updates;

        self
    }


    pub fn unset_update(
        &mut self,
        column: T,
    ) -> &mut Self {
        self.updates.shift_remove(
            &column,
        );

        self
    }


    pub fn merge_updates(
        &mut self,
        updates: IndexMap<T, SimpleExpr>,
    ) -> &mut Self {
        for (column, value) in updates {
            self.updates.insert(
                column,
                value,
            );
        }

        self
    }


    // =========================================
    // EXPRESSIONS
    // =========================================

    pub fn add_expression(
        &mut self,
        expression: SimpleExpr,
    ) -> &mut Self {
        self.expressions.push(
            expression,
        );

        self
    }

    // =========================================
    // GROUP BY
    // =========================================

    pub fn group_by(
        &mut self,
        column: T,
    ) -> &mut Self {
        self.group_by.push(column);

        self
    }

    // =========================================
    // ORDER BY
    // =========================================

    pub fn order_by(
        &mut self,
        column: T,
        direction: Order,
    ) -> &mut Self {
        self.order_by.push((
            column,
            direction,
        ));

        self
    }

    // =========================================
    // LIMIT
    // =========================================

    pub fn limit(
        &mut self,
        limit: u64,
    ) -> &mut Self {
        self.limit = Some(limit);

        self
    }

    // =========================================
    // SELECT BUILDER
    // =========================================

    pub fn select_builder(
        &self,
    ) -> SelectStatement {
        let mut statement =
            Query::select();

        statement.from(
            self.table.clone(),
        );

        // select
        if !self.select.is_empty()
            || !self.select_exprs.is_empty()
        {
            for select in self.select.clone() {
                statement.column(select);
            }

            for expr in
                self.select_exprs.clone()
            {
                statement.expr(expr);
            }
        } else {
            statement.column(
                ColumnRef::Asterisk,
            );
        }

        // where
        for (
            column,
            value,
        ) in self.columns.clone()
        {
            statement.and_where(
                Expr::col(column).eq(value),
            );
        }

        // expressions
        for expression in
            self.expressions.clone()
        {
            statement.and_where(
                expression,
            );
        }

        // order by
        for (
            column,
            direction,
        ) in self.order_by.clone()
        {
            statement.order_by(
                column,
                direction,
            );
        }

        // group by
        for group_by in
            self.group_by.clone()
        {
            statement.group_by_col(
                group_by,
            );
        }

        // limit
        if let Some(limit) = self.limit {
            statement.limit(limit);
        }

        statement
    }

    // =========================================
    // SQL
    // =========================================

    pub fn select_sql(&self) -> String {
        let (
            query,
            arguments,
        ) = self
            .select_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        let mut replace: Vec<String> =
            vec![];

        for argument in arguments.0.iter() {
            replace.push(format!(
                "{argument}"
            ));
        }

        Substrings::new(&query)
            .find_and_replace(
                "?",
                replace,
            )
            .to_string()
    }

    // =========================================
    // INSERT BUILDER
    // =========================================

    pub fn insert_builder(
        &self,
    ) -> InsertStatement {
        let mut statement =
            Query::insert();

        statement.into_table(
            self.table.clone(),
        );

        statement.columns(
            self.columns
                .clone()
                .into_keys()
                .collect::<Vec<_>>(),
        );

        statement.values_panic(
            self.columns
                .clone()
                .into_values()
                .collect::<Vec<_>>(),
        );

        statement.to_owned()
    }


    // =========================================
    // UPDATE BUILDER
    // =========================================

    pub fn update_builder(
        &self,
    ) -> UpdateStatement {
        let mut statement =
            Query::update();

        statement.table(
            self.table.clone(),
        );

        // where
        for (
            column,
            value,
        ) in self.columns.clone()
        {
            statement.and_where(
                Expr::col(column).eq(value),
            );
        }

        // expressions
        for expression in
            self.expressions.clone()
        {
            statement.and_where(
                expression,
            );
        }

        // update values
        let values = self
            .updates
            .clone()
            .into_iter()
            .collect::<Vec<_>>();

        statement.values(values);

        statement.to_owned()
    }

    // =========================================
    // DELETE BUILDER
    // =========================================

    pub fn delete_builder(
        &self,
    ) -> DeleteStatement {
        let mut statement =
            Query::delete();

        statement.from_table(
            self.table.clone(),
        );

        let mut condition =
            Cond::all();

        for (
            column,
            value,
        ) in self.columns.clone()
        {
            condition =
                condition.add(
                    Expr::col(column)
                        .eq(value),
                );
        }

        for expression in
            self.expressions.clone()
        {
            condition =
                condition.add(
                    expression,
                );
        }

        statement.cond_where(
            condition,
        );

        statement
    }

    // =========================================
    // FETCH
    // =========================================

    pub async fn fetch<'a, F>(
        &self,
    ) -> Result<Vec<F>, sqlx::Error>
    where
        F: for<'r> sqlx::FromRow<
                'r,
                MySqlRow,
            > + Send
            + Unpin,
    {
        let (
            query,
            arguments,
        ) = self
            .select_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        info!("SQL: {query}");

        sqlx::query_as_with::<
            '_,
            MySql,
            F,
            _,
        >(
            &query,
            arguments,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn fetch_one<'a, F>(
        &self,
    ) -> Result<F, sqlx::Error>
    where
        F: for<'r> sqlx::FromRow<
                'r,
                MySqlRow,
            > + Send
            + Unpin,
    {
        let (
            query,
            arguments,
        ) = self
            .select_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        info!("SQL: {query}");

        sqlx::query_as_with::<
            '_,
            MySql,
            F,
            _,
        >(
            &query,
            arguments,
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn fetch_optional<'a, F>(
        &self,
    ) -> Result<Option<F>, sqlx::Error>
    where
        F: for<'r> sqlx::FromRow<
                'r,
                MySqlRow,
            > + Send
            + Unpin,
    {
        let (
            query,
            arguments,
        ) = self
            .select_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        info!("SQL: {query}");

        sqlx::query_as_with::<
            '_,
            MySql,
            F,
            _,
        >(
            &query,
            arguments,
        )
        .fetch_optional(
            &self.pool,
        )
        .await
    }

    // =========================================
    // EXISTS
    // =========================================

    pub async fn exists(
        &self,
    ) -> bool {
        let (
            query,
            binds,
        ) = self
            .select_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        sqlx::query_with(
            &query,
            binds,
        )
        .fetch_one(&self.pool)
        .await
        .is_ok()
    }

    // =========================================
    // INSERT
    // =========================================

    pub async fn insert(
        &self,
    ) -> Result<&Self, sqlx::Error> {
        let (
            query,
            binds,
        ) = self
            .insert_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        sqlx::query_with(
            &query,
            binds,
        )
        .execute(&self.pool)
        .await
        .map_or_else(
            |e| Err(e),
            |_| Ok(self),
        )
    }

    // =========================================
    // UPDATE
    // =========================================

    pub async fn update(
        &self,
    ) -> Result<&Self, sqlx::Error> {
        let (
            query,
            binds,
        ) = self
            .update_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        sqlx::query_with(
            &query,
            binds,
        )
        .execute(&self.pool)
        .await
        .map_or_else(
            |e| Err(e),
            |_| Ok(self),
        )
    }


    // =========================================
    // INSERT ONCE
    // =========================================

    pub async fn insert_once(
        &self,
    ) -> Result<&Self, sqlx::Error> {
        if !self.exists().await {
            self.insert()
                .await
                .map_or_else(
                    |e| Err(e),
                    |_| Ok(self),
                )
        } else {
            Ok(self)
        }
    }

    // =========================================
    // DELETE
    // =========================================

    pub async fn delete(
        &self,
    ) -> Result<&Self, sqlx::Error> {
        let (
            query,
            binds,
        ) = self
            .delete_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        sqlx::query_with(
            &query,
            binds,
        )
        .execute(&self.pool)
        .await
        .map_or_else(
            |e| Err(e),
            |_| Ok(self),
        )
    }


    // =========================================
    // DELETE DUPLICATES
    // =========================================

    pub async fn delete_duplicates(
        &self,
        primary_key: T,
    ) -> Result<u32, sqlx::Error> {

        let mut crud = self.clone();

        crud.select_column(
            primary_key.clone(),
        );

        let mut deleted: u32 = 0;

        for (index, (id,)) in crud
            .fetch::<(i32,)>()
            .await?
            .into_iter()
            .enumerate()
        {
            if index > 0 {

                Self::new(
                    self.table.clone(),
                    &self.pool,
                )
                .set_column(
                    primary_key.clone(),
                    id.into(),
                )
                .delete()
                .await?;

                deleted += 1;
            }
        }

        Ok(deleted)
    }



    // =========================================
    // UNIQUE HEX
    // =========================================

    pub async fn unique_hex(
        &mut self,
        field_name: T,
        length: usize,
    ) -> Result<String, sqlx::Error> {
        let mut value =
            random_hex(length);

        self.set_column(
            field_name.clone(),
            value.clone().into(),
        );

        while self
            .fetch::<(String,)>()
            .await?
            .len()
            > 0
        {
            value = random_hex(length);

            self.set_column(
                field_name.clone(),
                value.clone().into(),
            );
        }

        Ok(value)
    }
}







//
// ============================================================
// JOIN CRUD
// ============================================================


#[derive(Clone)]
pub enum JoinColumn<T, S>
where
    T: Iden + Clone,
    S: Iden + Clone,
{
    Left(T),
    Right(S),
}

#[derive(Clone)]
pub struct JoinCrud<T, S>
where
    T: Iden + Clone,
    S: Iden + Clone,
{
    pub left_table: T,
    pub right_table: S,

    pub select: Vec<JoinColumn<T, S>>,
    pub select_exprs: Vec<SimpleExpr>,

    pub left_columns: IndexMap<T, SimpleExpr>,
    pub right_columns: IndexMap<S, SimpleExpr>,

    expressions: Vec<SimpleExpr>,

    group_by: Vec<JoinColumn<T, S>>,
    order_by:
        Vec<(JoinColumn<T, S>, Order)>,

    joins: Vec<(
        JoinType,
        TableRef,
        Condition,
    )>,

    limit: Option<u64>,

    pool: sqlx::pool::Pool<MySql>,
}

impl<T, S> JoinCrud<T, S>
where
    T: Iden
        + Clone
        + 'static
        + core::hash::Hash
        + Eq
        + PartialEq,
    S: Iden
        + Clone
        + 'static
        + core::hash::Hash
        + Eq
        + PartialEq,
{
    pub fn new(
        left_table: T,
        right_table: S,
        pool: &sqlx::pool::Pool<MySql>,
    ) -> Self {
        Self {
            left_table,
            right_table,

            select: vec![],
            select_exprs: vec![],

            left_columns: indexmap! {},
            right_columns: indexmap! {},

            expressions: vec![],

            group_by: vec![],
            order_by: vec![],

            joins: vec![],

            limit: None,

            pool: pool.clone(),
        }
    }

    // =========================================
    // SELECT
    // =========================================

    pub fn select_left(
        &mut self,
        column: T,
    ) -> &mut Self {
        self.select.push(
            JoinColumn::Left(column),
        );

        self
    }

    pub fn select_right(
        &mut self,
        column: S,
    ) -> &mut Self {
        self.select.push(
            JoinColumn::Right(column),
        );

        self
    }

    // =========================================
    // WHERE
    // =========================================

    pub fn set_left(
        &mut self,
        column: T,
        value: SimpleExpr,
    ) -> &mut Self {
        self.left_columns
            .insert(column, value);

        self
    }

    pub fn set_right(
        &mut self,
        column: S,
        value: SimpleExpr,
    ) -> &mut Self {
        self.right_columns
            .insert(column, value);

        self
    }

    // =========================================
    // EXPRESSIONS
    // =========================================

    pub fn add_expression(
        &mut self,
        expression: SimpleExpr,
    ) -> &mut Self {
        self.expressions.push(
            expression,
        );

        self
    }

    // =========================================
    // JOIN
    // =========================================

    pub fn inner_join(
        &mut self,
        condition: Condition,
    ) -> &mut Self {
        self.joins.push((
            JoinType::InnerJoin,
            TableRef::Table(self.right_table.clone().into_iden()),
            condition,
        ));

        self
    }

    // =========================================
    // SELECT BUILDER
    // =========================================

    pub fn select_builder(
        &self,
    ) -> SelectStatement {
        let mut statement =
            Query::select();

        statement.from(
            self.left_table.clone(),
        );

        // joins
        for (
            join_type,
            table,
            condition,
        ) in self.joins.clone()
        {
            statement.join(
                join_type,
                table,
                condition,
            );
        }

        // select
        if !self.select.is_empty() {
            for select in self.select.clone() {
                match select {
                    JoinColumn::Left(col) => {
                        statement.column((
                            self.left_table.clone(),
                            col,
                        ));
                    }

                    JoinColumn::Right(col) => {
                        statement.column((
                            self.right_table.clone(),
                            col,
                        ));
                    }
                }
            }
        } else {
            statement.column(
                ColumnRef::Asterisk,
            );
        }

        // left where
        for (
            column,
            value,
        ) in self.left_columns.clone()
        {
            statement.and_where(
                Expr::col((
                    self.left_table.clone(),
                    column,
                ))
                .eq(value),
            );
        }

        // right where
        for (
            column,
            value,
        ) in self.right_columns.clone()
        {
            statement.and_where(
                Expr::col((
                    self.right_table.clone(),
                    column,
                ))
                .eq(value),
            );
        }

        // expressions
        for expression in
            self.expressions.clone()
        {
            statement.and_where(
                expression,
            );
        }

        // group by
        for group_by in self.group_by.clone() {
            match group_by {
                JoinColumn::Left(col) => {
                    statement.group_by_col((
                        self.left_table.clone(),
                        col,
                    ));
                }

                JoinColumn::Right(col) => {
                    statement.group_by_col((
                        self.right_table.clone(),
                        col,
                    ));
                }
            }
        }

        // order by
        for (column, direction) in self.order_by.clone() {
            match column {
                JoinColumn::Left(col) => {
                    statement.order_by(
                        (
                            self.left_table.clone(),
                            col,
                        ),
                        direction,
                    );
                }

                JoinColumn::Right(col) => {
                    statement.order_by(
                        (
                            self.right_table.clone(),
                            col,
                        ),
                        direction,
                    );
                }
            }
        }

        // limit
        if let Some(limit) = self.limit {
            statement.limit(limit);
        }

        statement
    }

    // =========================================
    // FETCH
    // =========================================

    pub async fn fetch<'a, F>(
        &self,
    ) -> Result<Vec<F>, sqlx::Error>
    where
        F: for<'r> sqlx::FromRow<
                'r,
                MySqlRow,
            > + Send
            + Unpin,
    {
        let (
            query,
            arguments,
        ) = self
            .select_builder()
            .build_sqlx(
                MysqlQueryBuilder,
            );

        info!("SQL: {query}");

        sqlx::query_as_with::<
            '_,
            MySql,
            F,
            _,
        >(
            &query,
            arguments,
        )
        .fetch_all(&self.pool)
        .await
    }

    // =========================================
    // GROUP BY
    // =========================================

    pub fn group_by_left(
        &mut self,
        column: T,
    ) -> &mut Self {
        self.group_by.push(
            JoinColumn::Left(column),
        );

        self
    }

    pub fn group_by_right(
        &mut self,
        column: S,
    ) -> &mut Self {
        self.group_by.push(
            JoinColumn::Right(column),
        );

        self
    }

    // =========================================
    // ORDER BY
    // =========================================

    pub fn order_by_left(
        &mut self,
        column: T,
        direction: Order,
    ) -> &mut Self {
        self.order_by.push((
            JoinColumn::Left(column),
            direction,
        ));

        self
    }

    pub fn order_by_right(
        &mut self,
        column: S,
        direction: Order,
    ) -> &mut Self {
        self.order_by.push((
            JoinColumn::Right(column),
            direction,
        ));

        self
    }

    // =========================================
    // LIMIT
    // =========================================

    pub fn limit(
        &mut self,
        limit: u64,
    ) -> &mut Self {
        self.limit = Some(limit);

        self
    }
}

