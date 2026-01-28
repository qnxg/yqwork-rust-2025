use futures::future;

use crate::service::weihuda::feedback::FeedbackStatus;
use crate::service::weihuda::jifen::GoodsRecordStatus;
use crate::service::weihuda::zhihu::ZhihuStatus;
use crate::{result::AppResult, service};
use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct StatisticsItem {
    pub status: u32,
    pub count: u32,
}

pub async fn get_statistics() -> AppResult<HashMap<String, Vec<StatisticsItem>>> {
    let mut res = HashMap::new();
    // feedback
    let feedback_status = [
        FeedbackStatus::Closed,
        FeedbackStatus::Pending,
        FeedbackStatus::InProgress,
        FeedbackStatus::Resolving,
    ];
    let feedback_res = future::try_join_all(feedback_status.iter().map(|status| {
        service::weihuda::feedback::get_feedback_list(1, 1, None, Some(*status), None, None)
    }))
    .await?;
    res.insert(
        String::from("feedback"),
        feedback_res
            .into_iter()
            .enumerate()
            .map(|(i, (count, _))| StatisticsItem {
                status: feedback_status[i] as u32,
                count,
            })
            .collect(),
    );
    // goods_record
    let goods_record_status = [
        GoodsRecordStatus::Pending,
        GoodsRecordStatus::Exchanged,
        GoodsRecordStatus::Received,
    ];
    let goods_record_res = future::try_join_all(goods_record_status.iter().map(|status| {
        service::weihuda::jifen::get_goods_record_list(1, 1, None, None, Some(*status))
    }))
    .await?;
    res.insert(
        String::from("goods-record"),
        goods_record_res
            .into_iter()
            .enumerate()
            .map(|(i, (count, _))| StatisticsItem {
                status: goods_record_status[i] as u32,
                count,
            })
            .collect(),
    );
    // zhihu
    let zhihu_status = [
        ZhihuStatus::Pending,
        ZhihuStatus::Accepted,
        ZhihuStatus::Rejected,
    ];
    let zhihu_res = future::try_join_all(zhihu_status.iter().map(|status| {
        service::weihuda::zhihu::get_zhihu_list(1, 10, None, None, Some(*status), None)
    }))
    .await?;
    res.insert(
        String::from("zhihu"),
        zhihu_res
            .into_iter()
            .enumerate()
            .map(|(i, (count, _))| StatisticsItem {
                status: zhihu_status[i] as u32,
                count,
            })
            .collect(),
    );
    Ok(res)
}
