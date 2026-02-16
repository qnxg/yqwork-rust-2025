pub use crate::infra::mysql::jifen::{
    GoodsRecordStatus, delete_goods_record, get_goods_record, get_goods_record_list,
};
use crate::{infra, utils};

pub use crate::infra::mysql::jifen::{add_goods, delete_goods, get_goods_list, update_goods};

pub use crate::infra::mysql::jifen::{get_record, get_record_list};

pub use crate::infra::mysql::jifen::{add_rule, delete_rule, get_rule_list, update_rule};

use crate::result::AppResult;
use crate::service::qnxg::user::User;

pub async fn add_record(
    update_by: &User,
    stu_id: &str,
    delta: i32,
    reason: &str,
) -> AppResult<u32> {
    let res =
        infra::mysql::jifen::add_record("rengong", &update_by.info.name, stu_id, reason, delta)
            .await?;
    infra::mysql::jifen::update_jifen(stu_id, delta).await?;
    Ok(res)
}

pub async fn receive_goods(id: u32) -> AppResult<()> {
    infra::mysql::jifen::update_goods_record(
        id,
        GoodsRecordStatus::Received,
        Some(utils::now_time()),
    )
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_record_list() {
        let list = get_record_list(1, 10, None, None, None).await.unwrap();
        println!("{:#?}", list);
    }
}
