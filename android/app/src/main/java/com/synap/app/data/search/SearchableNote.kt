package com.synap.app.data.search

import androidx.appsearch.annotation.Document
import androidx.appsearch.app.AppSearchSchema.StringPropertyConfig

@Document
data class SearchableNote(
    // 命名空间，比如 "user_notes"
    @Document.Namespace
    val namespace: String,

    // 笔记的唯一 ID，和数据库里的 ID 对应
    @Document.Id
    val id: String,

    // 搜索权重，数值越高在系统搜索结果里越靠前
    @Document.Score
    val score: Int,

    // 笔记正文，设置为支持前缀搜索（INDEXING_TYPE_PREFIXES）
    @Document.StringProperty(indexingType = StringPropertyConfig.INDEXING_TYPE_PREFIXES)
    val content: String,

    // 笔记的时间戳，可以用于排序
    @Document.LongProperty
    val creationTimestampMillis: Long
)