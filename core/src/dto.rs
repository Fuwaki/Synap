use serde::{Deserialize, Serialize};

/// 绝对纯净的、跨端通用的 DTO
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")] // 照顾 TS 和 Kotlin 的命名习惯
pub struct NoteDTO {
    pub id: String, // Uuid 转成标准的 36 位字符串
    // pub short_id: String, // 8位 NanoID
    pub content: String,
    pub tags: Vec<String>, // 直接给文字，前端不关心 Tag 的内部 UUID
    pub created_at: u64,   // 毫秒时间戳
    pub deleted: bool,
}
